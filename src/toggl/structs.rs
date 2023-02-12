use std::collections::HashSet;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct TimeEntry {
    pub id: i64,
    pub user_id: i64,
    pub workspace_id: i64,
    pub at: String,
    pub description: String,
    pub duration: i64,
    pub duronly: bool,
    pub start: Option<String>,
    pub stop: Option<String>,
    pub server_deleted_at: Option<String>,
    pub tags: Option<HashSet<String>>,
}

#[derive(Serialize, Debug)]
pub struct TagEntry {
    pub op: String,
    pub path: String,
    pub value: HashSet<String>,
}
#[derive(Serialize, Debug)]
pub struct BatchRequest {
    pub array: Vec<TagEntry>,
}
