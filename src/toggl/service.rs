use std::collections::HashSet;

use chrono::{Days, NaiveDate};
use reqwest::{header::CONTENT_TYPE, Client, Method};

use super::structs::{BatchRequest, TagEntry, TimeEntry};

const TIME_URL: &str = "https://api.track.toggl.com/api/v9/me/time_entries";
const BATCH_URL: &str =
    "https://api.track.toggl.com/api/v9/workspaces/{workspace_id}/time_entries/{time_entry_ids}";

pub async fn retrieve_entries(
    client: &Client,
    username: String,
    password: String,
    start_date: NaiveDate,
) -> anyhow::Result<Vec<TimeEntry>> {
    let start_string = start_date.format("%Y-%m-%d").to_string();
    let end_string = start_date
        .checked_add_days(Days::new(1))
        .expect("Should never overflow?")
        .format("%Y-%m-%d")
        .to_string();

    let available_entries = client
        .request(Method::GET, TIME_URL)
        .query(&[("start_date", start_string), ("end_date", end_string)])
        .header(CONTENT_TYPE, "application/json")
        .basic_auth(username, Some(password))
        .send()
        .await?
        .json::<Vec<TimeEntry>>()
        .await?;

    Ok(available_entries)
}

pub async fn try_tag_entries(
    client: &Client,
    username: String,
    password: String,
    entries: Vec<TimeEntry>,
) -> anyhow::Result<()> {
    let workspace_id = entries.get(0).expect("Atleast one present").workspace_id;
    let time_entry_ids = entries
        .iter()
        .map(|entry| entry.id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let tag_entries = entries
        .iter()
        .map(|entry| {
            if let Some(existing_tags) = &entry.tags {
                let mut tags = HashSet::new();
                tags.insert("Synced".to_string());
                for tag in existing_tags.iter() {
                    tags.insert(tag.to_string());
                }
                TagEntry {
                    value: tags,
                    op: "replace".to_string(),
                    path: "/tags".to_string(),
                }
            } else {
                let mut tags = HashSet::new();
                tags.insert("Synced".to_string());
                TagEntry {
                    value: tags,
                    op: "replace".to_string(),
                    path: "/tags".to_string(),
                }
            }
        })
        .collect::<Vec<TagEntry>>();
    let batch_request = BatchRequest { array: tag_entries };
    client
        .request(
            Method::PATCH,
            BATCH_URL
                .replace("workspace_id", &workspace_id.to_string())
                .replace("time_entry_ids", &time_entry_ids),
        )
        .header(CONTENT_TYPE, "application/json")
        .basic_auth(username, Some(password))
        .json(&batch_request)
        .send()
        .await?;

    Ok(())
}