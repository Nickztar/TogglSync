use reqwest::{header::CONTENT_TYPE, Client, Method};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeEntry {
    pub id: i64,
    pub user_id: i64,
    pub at: String,
    pub description: String,
    pub duration: i64,
    pub duronly: bool,
    pub start: Option<String>,
    pub stop: Option<String>,
    pub server_deleted_at: Option<String>,
}

const TOGGLE_URL: &str = "https://api.track.toggl.com/api/v9/me/time_entries";

pub async fn retrieve_entries(
    client: &Client,
    username: String,
    password: String,
    since_unix_secs: u64,
) -> anyhow::Result<Vec<TimeEntry>> {
    let available_entries = client
        .request(
            Method::GET,
            TOGGLE_URL
        )
        .query(&[("since", since_unix_secs)])
        .header(CONTENT_TYPE, "application/json")
        .basic_auth(username, Some(password))
        .send()
        .await?
        .json::<Vec<TimeEntry>>()
        .await?;

    return Ok(available_entries);
}
