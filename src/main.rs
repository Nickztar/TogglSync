#![feature(let_chains)]
use crate::{
    storage::{
        credentials::retrieve_credentials,
        keys::{retreive_keys, store_keys},
    },
    tempo::service::{create_worklogs, datetime_to_date_and_time},
};
use chrono::{DateTime, Days, FixedOffset, NaiveDateTime, NaiveTime, Utc};
use humantime::format_duration;
use inquire::{Confirm, CustomUserError, Text};
use reqwest::Client;
use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};
use tempo::structs::Worklog;

#[macro_use]
extern crate savefile_derive;

use toggl::*;

mod storage;
mod tempo;
mod toggl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let credentials = retrieve_credentials()?;
    let mut available_keys = retreive_keys()?;
    let client = Client::new();
    let available_entries = retrieve_entries(
        &client,
        credentials.username,
        credentials.password,
        days_ago(1), //TODO: Ask for a specific day :)
    )
    .await?;

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
    );
    let mut accumulated_entries: Vec<Worklog> = Vec::new();
    for entry in available_entries
        .iter()
        .filter(|entry| entry.duration.is_positive() && entry.server_deleted_at.is_none())
    {
        let curr_keys = available_keys.clone();
        let duration = Duration::from_secs(entry.duration as u64);
        let start_string = entry.start.as_ref().unwrap();
        let start_datetime = DateTime::parse_from_rfc3339(start_string)?;
        let (start_date, start_time) = datetime_to_date_and_time(start_datetime);
        let confirm = Confirm::new(&format!(
            "{} with duration: {}, handle? (y/n) (ESC to sync)",
            entry.description,
            format_duration(duration)
        ))
        .with_default(true)
        .prompt_skippable()?;
        if let Some(should_handle) = confirm {
            if !should_handle {
                continue;
            } else {
                let possible_key_desc = entry.description.split_once(':');
                if let Some((key, description)) = possible_key_desc {
                    let new_key: String = Text::new("Key for this entry")
                        .with_autocomplete(move |key: &str| {
                            key_suggestor(key.to_string(), &curr_keys)
                        })
                        .with_default(key)
                        .prompt()?;
                    let new_desc = Text::new("Description for this entry")
                        .with_default(description)
                        .prompt()?;
                    available_keys.insert(new_key.to_string());
                    let worklog = Worklog {
                        author_account_id: credentials.account_id.to_string(),
                        description: new_desc,
                        issue_key: new_key,
                        start_date,
                        start_time,
                        time_spent_seconds: duration.as_secs(),
                        date: start_datetime,
                    };
                    accumulated_entries.push(worklog);
                } else {
                    let new_key: String = Text::new("Key for this entry")
                        .with_autocomplete(move |key: &str| {
                            key_suggestor(key.to_string(), &curr_keys)
                        })
                        .prompt()?;
                    let new_desc = Text::new("Description for this entry")
                        .with_default(&entry.description)
                        .prompt()?;
                    available_keys.insert(new_key.to_string());
                    let worklog = Worklog {
                        author_account_id: credentials.account_id.to_string(),
                        description: new_desc,
                        issue_key: new_key,
                        start_date,
                        start_time,
                        time_spent_seconds: duration.as_secs(),
                        date: start_datetime,
                    };
                    accumulated_entries.push(worklog);
                }
            }
        } else {
            break;
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
            let (start_date, start_time) = datetime_to_date_and_time(start_datetime);
            let merged_description = group
                .iter()
                .map(|entry| entry.description.to_string())
                .fold(String::new(), |acc, desc| {
                    //Descriptions are either added together with &
                    //Or if they are already contained in the string they are ignored.
                    if acc.contains(&desc) {
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

// let possible_issue_key = entry.description.split_once(":");
// if let Some((key, description)) = possible_issue_key && let Some(start_date) = entry.start && let Some(end_date) = entry.stop {
//     let exists = worklogs.iter().find(|worklog| {
//         worklog.issue.key == key && worklog.description == description && start_date == worklog.
//     })
// }

/// This could be faster by using smarter ways to check for matches, when dealing with larger datasets.
fn key_suggestor(
    input: String,
    keys: &HashSet<String>,
) -> std::result::Result<Vec<String>, CustomUserError> {
    let input = input.to_lowercase();

    Ok(keys
        .iter()
        .filter(|p| p.to_lowercase().contains(&input))
        .take(5)
        .map(|p| String::from(p))
        .collect())
}

fn days_ago(days: u64) -> i64 {
    let now = Utc::now();
    let days_ago = now
        .checked_sub_days(Days::new(days - 1))
        .expect("Should never overflow?");
    let date = days_ago.date_naive();
    let time = NaiveTime::from_hms_opt(0, 0, 0).expect("Static parsable");
    let datetime = NaiveDateTime::new(date, time);
    datetime.timestamp()
}
