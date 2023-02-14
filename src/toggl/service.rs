use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Days, NaiveDate, Utc};
use reqwest::{header::CONTENT_TYPE, Client, Method};

use super::structs::{BatchRequest, MergedEntry, TagEntry, TimeEntry};

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

pub fn merge_filter_entries(entries: Vec<TimeEntry>) -> Vec<MergedEntry> {
    let grouped_entries: BTreeMap<String, Vec<TimeEntry>> =
        entries.into_iter().fold(BTreeMap::new(), |mut acc, entry| {
            //Filter out deleted and non-finished entries
            if entry.duration.is_positive() && entry.server_deleted_at.is_none() {
                //Merge on description (TODO: Maybe also merge on projects/tags etc, just like toggl)
                acc.entry(entry.description.to_string())
                    .or_default()
                    .push(entry);
            }
            acc
        });
    let mut merged_entries: Vec<MergedEntry> = Vec::new();
    for (description, group) in grouped_entries {
        let first = group.get(0);
        if let Some(first_entry) = first {
            let group_len = group.len();
            let mut tags: Vec<_> = Vec::with_capacity(group_len);
            let mut start_time: DateTime<Utc> = Utc::now();
            let mut duration = 0i64;
            for entry in group.iter() {
                duration += entry.duration;
                tags.push((entry.id, entry.tags.clone()));
                if let Some(start) = entry.start {
                    start_time = start_time.min(start.with_timezone(&Utc))
                }
            }
            let merged = MergedEntry {
                user_id: first_entry.user_id,
                workspace_id: first_entry.workspace_id,
                duration,
                description,
                start: start_time,
                tags,
            };
            merged_entries.push(merged);
        } else {
            continue;
        }
    }

    merged_entries
}

#[allow(dead_code)]
pub async fn try_tag_entries(
    client: &Client,
    username: String,
    password: String,
    entries: Vec<TimeEntry>,
    new_tag: &str,
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
                tags.insert(new_tag.to_string());
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
                tags.insert(new_tag.to_string());
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
