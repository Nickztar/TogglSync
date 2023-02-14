use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Debug)]
pub struct Worklog {
    #[serde(rename = "authorAccountId")]
    pub author_account_id: String,
    pub description: String,
    #[serde(rename = "issueKey")]
    pub issue_key: String,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent_seconds: u64,
    #[serde(skip_serializing)]
    pub date: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Issue {
    pub key: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkLogResult {
    pub issue: Issue,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent_seconds: u64,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WorkLogResponse {
    results: Vec<WorkLogResult>
}