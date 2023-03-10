use std::collections::HashSet;

use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Deserializer, Serialize};
#[derive(Deserialize, Debug)]
pub struct TimeEntry {
    pub id: i64,
    pub user_id: i64,
    pub workspace_id: i64,
    pub at: String,
    pub description: String,
    pub duration: i64,
    pub duronly: bool,
    #[serde(deserialize_with = "date_time_from_str")]
    pub start: Option<DateTime<FixedOffset>>,
    pub stop: Option<String>,
    pub server_deleted_at: Option<String>,
    pub tags: Option<HashSet<String>>,
}

pub struct MergedEntry {
    pub user_id: i64,
    pub workspace_id: i64,
    pub duration: i64,
    pub description: String,
    pub start: DateTime<Utc>,
    pub tags: Vec<EntryTag>,
}

#[derive(Clone)]
pub struct EntryTag {
    pub id: i64,
    pub user_id: i64,
    pub workspace_id: i64,
    pub tags: Option<HashSet<String>>,
}

fn date_time_from_str<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let parsed = DateTime::parse_from_rfc3339(&s);
    match parsed {
        Ok(date) => Ok(Some(date)),
        Err(_) => Ok(None),
    }
}

#[derive(Serialize, Debug)]
pub struct TagRequest {
    pub tags: HashSet<String>,
}
