use std::fs::remove_file;

use inquire::{Confirm, Password, Text};
use savefile::{load_file, save_file};
use savefile_derive::Savefile;

#[derive(Savefile, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub tempo_token: String,
    pub account_id: String,
}

const CRED_FILE: &str = "toggl_sync.bin";

pub fn retrieve_credentials() -> anyhow::Result<Credentials> {
    let existing = load_file::<Credentials, _>(CRED_FILE, 0);

    if let Ok(credentials) = existing {
        //TODO: Decrypt pls
        return Ok(credentials);
    }

    let username: String = Text::new("Toggl Username").prompt()?;
    //TODO: Encrypt pls
    let password = Password::new("Toggl Password").prompt()?;
    let tempo_token: String = Text::new("Tempo token").with_help_message("https://effectsoft.atlassian.net/plugins/servlet/ac/io.tempo.jira/tempo-app#!/configuration/api-integration").prompt()?; //TODO: Link to how to create
    let account_id: String = Text::new("Jira AccountId").with_help_message("Click your Profile menu in the upper-right, then select \"Profile\". In the URL after /people/ is your account ID.").prompt()?; //TODO: Link to how to retrieve

    let credentials = Credentials {
        username,
        password,
        tempo_token,
        account_id,
    };
    if Confirm::new("Stay logged in? (y/n)").prompt()? {
        let save_result = save_file(CRED_FILE, 0, &credentials);
        if save_result.is_err() {
            println!("Failed to save credentials :(");
        } else {
            println!("Ok, I will remember them!");
        }
    } else {
        println!("Ok, I will not save them!");
    }

    Ok(credentials)
}

pub fn clear_credentials() -> anyhow::Result<()> {
    remove_file(CRED_FILE)?;
    Ok(())
}
