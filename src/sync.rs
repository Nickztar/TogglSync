use std::{collections::HashMap, time::Duration};

use crate::{
    storage::{
        credentials::retrieve_credentials,
        keys::{retreive_keys, store_keys},
    },
    tempo::{
        service::{create_worklogs, datetime_to_date_and_time},
        structs::Worklog,
    },
    toggl::{
        issue_completer::IssueCompleter,
        service::{merge_filter_entries, retrieve_entries},
        structs::MergedEntry,
    },
    utils::{clean_description, clean_key},
};
use anyhow::Ok;
use chrono::{Utc, Weekday};
use colored::Colorize;
use humantime::format_duration;
use inquire::{Confirm, DateSelect, Text};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;

lazy_static! {
    static ref RE: Regex = Regex::new(r"\b[A-Z][A-Z0-9_]+-[1-9][0-9]*").unwrap();
}

pub async fn sync_toggle() -> anyhow::Result<()> {
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
    let initial_len = available_entries.len();
    println!("Found {} Toggl entries", initial_len.to_string().blue());
    let merged_entries = merge_filter_entries(available_entries);
    println!(
        "Merged entries into: {}",
        merged_entries.len().to_string().red()
    );
    let total_duration = merged_entries
        .iter()
        .fold(0u64, |duration, entry| duration + (entry.duration as u64));
    println!(
        "Found a total duration of {}, looping now: ",
        format_duration(Duration::from_secs(total_duration))
            .to_string()
            .blue()
            .underline()
    );
    let mut accumulated_entries: Vec<Worklog> = Vec::new();
    for entry in merged_entries.iter() {
        let curr_keys = available_keys.clone();
        let duration = Duration::from_secs(entry.duration as u64);
        let start_datetime = entry.start;
        let (start_date, start_time) = datetime_to_date_and_time(&start_datetime);
        let (key, desc) = get_key_desc(entry, curr_keys)?;
        if !available_keys.contains_key(&key) {
            let key_desc = Text::new(&format!("{}, description?", key.to_string().blue()))
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
            date: start_datetime,
        };
        accumulated_entries.push(worklog);
    }

    let _failed = create_worklogs(credentials.tempo_token.to_string(), accumulated_entries).await?;
    //TODO: Allow fixing these
    store_keys(available_keys)?;

    Ok(())
}

fn get_key_desc(
    entry: &MergedEntry,
    curr_keys: HashMap<String, String>,
) -> anyhow::Result<(String, String)> {
    let possible_key = RE.captures(&entry.description);
    let mut key: Option<String> = None;
    let mut desc: String = entry.description.to_string();
    let duration = Duration::from_secs(entry.duration as u64);
    let mut edit_requested = false;
    if let Some(captures) = possible_key && let Some(key_match) = captures.get(0) {
        let possible_key = key_match.as_str();
        desc = clean_description(&entry.description.replace(key_match.as_str(), ""));
        key = Some(possible_key.to_string());
    }
    if let Some(pos_key) = &key {
        edit_requested =
            Confirm::new(&format!("{}: {}. Edit? (y/n)", pos_key.red(), desc.green())).prompt()?;
    } else {
        println!(
            "Missing key! Desc: {}, Duration: {}",
            entry.description.red().underline(),
            format_duration(duration).to_string().blue().underline()
        );
        key = Some(
            Text::new("Key?")
                .with_autocomplete(IssueCompleter::new(curr_keys.clone()))
                .prompt()?,
        );
    }
    if edit_requested {
        desc = Text::new("Description?").with_default(&desc).prompt()?;
    }
    match key {
        Some(new_key) => {
            let clean_key = clean_key(&new_key.to_owned());
            Ok((clean_key, desc))
        }
        None => get_key_desc(entry, curr_keys),
    }
}
