#![feature(let_chains)]
use crate::{
    storage::{
        credentials::retrieve_credentials,
        keys::{retreive_keys, store_keys},
    },
    tempo::service::{create_worklogs, datetime_to_date_and_time},
    toggl::{service::retrieve_entries, issue_completer::IssueCompleter},
};
use chrono::Weekday;
use chrono::{DateTime, FixedOffset, Utc};
use colored::Colorize;
use humantime::format_duration;
use inquire::{Confirm, DateSelect, Text};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use std::{
    collections::BTreeMap,
    time::Duration,
};
use tempo::structs::Worklog;

#[macro_use]
extern crate savefile_derive;

mod storage;
mod tempo;
mod toggl;

const EFFECTSOFT_ASCII: &str = r"  ______ ______ ______ ______ _____ _______ _____  ____  ______ _______ 
|  ____|  ____|  ____|  ____/ ____|__   __/ ____|/ __ \|  ____|__   __|
| |__  | |__  | |__  | |__ | |       | | | (___ | |  | | |__     | |   
|  __| |  __| |  __| |  __|| |       | |  \___ \| |  | |  __|    | |   
| |____| |    | |    | |___| |____   | |  ____) | |__| | |       | |   
|______|_|    |_|    |______\_____|  |_| |_____/ \____/|_|       |_|
";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", EFFECTSOFT_ASCII.red());
    let credentials = retrieve_credentials()?;
    let mut available_keys = retreive_keys()?;
    let selected_date = DateSelect::new("What day do you want to sync?")
        .with_starting_date(Utc::now().date_naive())
        .with_week_start(Weekday::Mon)
        .prompt()?;

    let client = Client::new();
    let available_entries = retrieve_entries(
        &client,
        credentials.username,
        credentials.password,
        selected_date,
    )
    .await?;

    println!("Found {} Toggl entries", available_entries.len().to_string().blue().underline());
    
    //TODO: Check which entries are already in tempo
    let total_duration = available_entries.iter().fold(0u64, |duration, entry| {
        if entry.duration.is_positive() && entry.server_deleted_at.is_none() {
            duration + (entry.duration as u64)
        } else {
            duration
        }
    });
    println!(
        "Found a total duration of {}, looping now: ",
        format_duration(Duration::from_secs(total_duration))
            .to_string()
            .blue()
            .underline()
    );
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\b[A-Z][A-Z0-9_]+-[1-9][0-9]*").unwrap();
    }
    let mut accumulated_entries: Vec<Worklog> = Vec::new();
    for entry in available_entries
        .iter()
        .filter(|entry| entry.duration.is_positive() && entry.server_deleted_at.is_none())
    {
        let curr_keys = available_keys.clone();
        let duration = Duration::from_secs(entry.duration as u64);
        let start_datetime = entry.start.as_ref().expect("All entries have starttimes");
        let (start_date, start_time) = datetime_to_date_and_time(start_datetime);
        let possible_key = RE.captures(&entry.description);
        if let Some(captures) = possible_key && let Some(key_match) = captures.get(0) {
            let possible_key = key_match.as_str();
            let mut desc = clean_description(&entry.description.replace(key_match.as_str(), ""));
            let edit_requested = Confirm::new(&format!("Entry with key: {} and description: {}. Edit? (y/n)", possible_key.red().underline(), desc.green().underline())).prompt()?;
            
            let mut key = possible_key.to_string();
            if edit_requested {
                key = Text::new("Key for this entry")
                    .with_autocomplete(IssueCompleter::new(curr_keys))
                    .with_default(possible_key)
                    .prompt()?;
                desc = Text::new("Description for this entry")
                    .with_default(&desc)
                    .prompt()?;
            }
            let key = clean_key(&key);
            dbg!(&available_keys);
            if !available_keys.contains_key(&key) {
                let key_desc = Text::new(&format!("{}: is a new key, what is the description for it?", key.to_string().blue()))
                    .with_default(&desc)
                    .prompt()?;
                available_keys.insert(key.to_string(), key_desc);
            }
            let worklog = Worklog {
                author_account_id: credentials.account_id.to_string(),
                description: desc,
                issue_key: key,
                start_date,
                start_time,
                time_spent_seconds: duration.as_secs(),
                date: *start_datetime,
            };
            accumulated_entries.push(worklog);
        } else {
            println!("Missing key! Desc: {}, Duration: {}", entry.description.red().underline(), 
                format_duration(duration)
                    .to_string()
                    .blue()
                    .underline()
            );
            let new_key = Text::new("Key for this entry")
                .with_autocomplete(IssueCompleter::new(curr_keys))
                .prompt()?;
            let new_key = clean_key(&new_key);
            let new_desc = Text::new("Description for this entry")
                .with_default(&entry.description)
                .prompt()?;
            dbg!(&available_keys);
            if !available_keys.contains_key(&new_key) {
                let key_desc = Text::new(&format!("{} is a new key, what is the description for it?", new_key.blue()))
                    .with_default(&new_desc)
                    .prompt()?;
                available_keys.insert(new_key.to_string(), key_desc);
            }
            let worklog = Worklog {
                author_account_id: credentials.account_id.to_string(),
                description: new_desc.trim().to_string(),
                issue_key: new_key.trim().to_string(),
                start_date,
                start_time,
                time_spent_seconds: duration.as_secs(),
                date: *start_datetime,
            };
            accumulated_entries.push(worklog);
        }
    }

    let should_group = Confirm::new("Do you want me to group based on issues?").prompt()?;

    if should_group {
        //Group by key
        let grouped_entries: BTreeMap<String, Vec<Worklog>> =
            accumulated_entries
                .into_iter()
                .fold(BTreeMap::new(), |mut acc, entry| {
                    acc.entry(entry.issue_key.to_string())
                        .or_default()
                        .push(entry);
                    acc
                });
        let mut merged_entries: Vec<Worklog> = Vec::new();
        for (key, group) in grouped_entries {
            //Add together times
            let acc_duration: u64 = group.iter().map(|entry| entry.time_spent_seconds).sum();
            //Take the earliest start time
            let start_datetime: DateTime<FixedOffset> =
                group.iter().map(|entry| entry.date).min().unwrap();
            let (start_date, start_time) = datetime_to_date_and_time(&start_datetime);
            let merged_description = group
                .iter()
                .map(|entry| entry.description.to_string())
                .fold(String::new(), |acc, desc| {
                    //Descriptions are either added together with &
                    //Or if they are already contained in the string they are ignored.
                    if acc.contains(&desc) || desc.is_empty() {
                        acc
                    } else {
                        format!("{} & {}", acc, desc)
                    }
                });
            let merged_log = Worklog {
                author_account_id: credentials.account_id.to_string(),
                description: merged_description,
                date: start_datetime,
                issue_key: key,
                start_date,
                start_time,
                time_spent_seconds: acc_duration,
            };
            merged_entries.push(merged_log);
        }
        let _ = create_worklogs(credentials.tempo_token.to_string(), merged_entries).await?;
    } else {
        let _ = create_worklogs(credentials.tempo_token.to_string(), accumulated_entries).await?;
    }

    store_keys(available_keys)?;
    Ok(())
}

fn clean_description(input: &str) -> String {
    let chars: &[_] = &[':', '-'];
    input.trim().trim_matches(chars).trim().to_string()
}

fn clean_key(input: &str) -> String {
    match input.split_once(' ') {
        Some((key, _)) => key.trim().to_string(),
        None => input.trim().to_string(),
    }
}
