use crate::credentials::retrieve_credentials;
use chrono::DateTime;
use humantime::format_duration;
use inquire::{Confirm, Text};
use reqwest::{Client, StatusCode};
use std::time::{Duration, SystemTime};

#[macro_use]
extern crate savefile_derive;

use tempo::*;
use toggl::*;

mod credentials;
mod tempo;
mod toggl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let credentials = retrieve_credentials()?;
    let client = Client::new();
    let available_entries = retrieve_entries(
        &client,
        credentials.username,
        credentials.password,
        days_ago(1), //TODO: Ask for a specific day :)
    )
    .await?;

    //TODO: Check which entries are already in tempo

    println!("Found {} entries, looping now: ", available_entries.len());
    let mut accumulated_entries: Vec<Worklog> = Vec::new();
    for entry in available_entries
        .iter()
        .filter(|entry| entry.duration.is_positive() && entry.server_deleted_at.is_none())
    {
        let duration = Duration::from_secs(entry.duration as u64);
        let start_string = entry.start.as_ref().unwrap();
        let start_datetime = DateTime::parse_from_rfc3339(&start_string)?;
        let start_date = start_datetime.format("%Y-%m-%d").to_string();
        let start_time = start_datetime.format("%H:%M:%S").to_string();
        let confirm = Confirm::new(&format!(
            "{} with duration: {}, skip? (ESC to sync)",
            entry.description,
            format_duration(duration)
        ))
        .prompt_skippable()?;
        if let Some(should_skip) = confirm {
            if should_skip {
                continue;
            } else {
                let possible_key_desc = entry.description.split_once(':');
                if let Some((key, description)) = possible_key_desc {
                    let new_key: String =
                        Text::new("Key for this entry").with_default(key).prompt()?;
                    let new_desc = Text::new("Description for this entry")
                        .with_default(description)
                        .prompt()?;

                    let worklog = Worklog {
                        author_account_id: credentials.account_id.to_string(),
                        description: new_desc,
                        issue_key: new_key,
                        start_date,
                        start_time,
                        time_spent_seconds: duration.as_secs(),
                    };
                    accumulated_entries.push(worklog);
                } else {
                    let new_key: String = Text::new("Key for this entry").prompt()?;
                    let new_desc = Text::new("Description for this entry")
                        .with_default(&entry.description)
                        .prompt()?;
                    let worklog = Worklog {
                        author_account_id: credentials.account_id.to_string(),
                        description: new_desc,
                        issue_key: new_key,
                        start_date,
                        start_time,
                        time_spent_seconds: duration.as_secs(),
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
        //Add together times
        //Take the earliest start time
        //Descriptions are either added together with &
        //Or if they are already contained in the string they are ignored.
    }

    //Loop over the remaining and ask for key, and possible description?
    let tempo_client = Client::new();
    for acc_entry in accumulated_entries {
        let issue_key = acc_entry.issue_key.to_string();
        let response = create_worklog(
            &tempo_client,
            credentials.tempo_token.to_string(),
            acc_entry,
        )
        .await;
        if let Ok(res) = response {
            if res.status() != StatusCode::OK {
                println!("{} failed to be added to tempo!", issue_key);
            } else {
                println!("{} was added to tempo!", issue_key);
            }
        } else {
            println!("{} failed to be added to tempo!", issue_key);
        }
    }
    Ok(())
}

// let possible_issue_key = entry.description.split_once(":");
// if let Some((key, description)) = possible_issue_key && let Some(start_date) = entry.start && let Some(end_date) = entry.stop {
//     let exists = worklogs.iter().find(|worklog| {
//         worklog.issue.key == key && worklog.description == description && start_date == worklog.
//     })
// }

fn days_ago(days: u64) -> u64 {
    let now = SystemTime::now();
    let time_at_days_ago = now
        .checked_sub(Duration::from_secs(60 * 60 * 24 * days))
        .unwrap(); //24 hours ago
    time_at_days_ago
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
