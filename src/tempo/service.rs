use anyhow::Ok;
use chrono::{DateTime, FixedOffset};
use reqwest::{Client, Method, Response, StatusCode};

use super::structs::Worklog;
const TEMPO_URL: &str = "https://api.tempo.io/core/3/worklogs";

pub fn datetime_to_date_and_time(date: &DateTime<FixedOffset>) -> (String, String) {
    let start_date = date.format("%Y-%m-%d").to_string();
    let start_time = date.format("%H:%M:%S").to_string();
    (start_date, start_time)
}

pub async fn create_worklogs(token: String, worklogs: Vec<Worklog>) -> anyhow::Result<Vec<String>> {
    let tempo_client = Client::new();
    let mut entry_hashes: Vec<String> = Vec::new();
    for log in worklogs {
        let issue_key = log.issue_key.to_string();
        let response = create_worklog(
            &tempo_client,
            token.to_string(),
            log,
        )
        .await;
        if let std::result::Result::Ok(res) = response {
            //TODO: Parse result on issue
            if res.status() != StatusCode::OK {
                println!("{} failed to be added to tempo!", issue_key);
            } else {
                println!("{} was added to tempo!", issue_key);
            }
        } else {
            println!("{} failed to be added to tempo!", issue_key);
        }
        entry_hashes.push(issue_key); //TODO: Figure out how to track what has already been done?
    }

    Ok(entry_hashes)
}

pub async fn create_worklog(client: &Client, token: String, work_log: Worklog) -> anyhow::Result<Response> {
    let response = client
        .request(Method::POST, TEMPO_URL)
        .json(&work_log)
        .bearer_auth(token)
        .send()
        .await?;
    dbg!(&response);
    Ok(response)
}

// pub async fn get_worklogs(client: &Client, token: String, account_id: String, from: String, to: String) -> anyhow::Result<Vec<WorkLogResult>> {
//     let worklogs = client
//         .request(Method::GET, TEMPO_URL.to_string() + "/user/" + &account_id)
//         .query(&[("limit", "1000".to_string()), ("from", from), ("to", to)])
//         .bearer_auth(token)
//         .send()
//         .await?
//         .json::<WorkLogResponse>()
//         .await?;

//     Ok(worklogs.results)
// }
