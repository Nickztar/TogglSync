use reqwest::{Client, Method, Response};
use serde::{Deserialize, Serialize};
const TEMPO_URL: &str = "https://api.tempo.io/core/3/worklogs";

#[derive(Serialize, Deserialize, Debug)]
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

pub async fn create_worklog(client: &Client, token: String, work_log: Worklog) -> anyhow::Result<Response> {
    let response = client
        .request(Method::POST, TEMPO_URL)
        .json(&work_log)
        .bearer_auth(token)
        .send()
        .await?;
    
    return Ok(response);
}

pub async fn get_worklogs(client: &Client, token: String, account_id: String, from: String, to: String) -> anyhow::Result<Vec<WorkLogResult>> {
    let worklogs = client
        .request(Method::GET, TEMPO_URL.to_string() + "/user/" + &account_id)
        .query(&[("limit", "1000".to_string()), ("from", from), ("to", to)])
        .bearer_auth(token)
        .send()
        .await?
        .json::<WorkLogResponse>()
        .await?;

    Ok(worklogs.results)
}
