use inquire::{Text, Password, Confirm};
use savefile::{load_file, save_file};


#[derive(Savefile, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub tempo_token: String,
    pub account_id: String
}

const CRED_FILE: &str = "toggl_sync.bin";

pub fn retrieve_credentials() -> anyhow::Result<Credentials> {
    let existing = load_file::<Credentials,_>(CRED_FILE, 0);

    let use_existing_creds = Confirm::new("Found existing credentials, should I use them? (y/n)");

    if let Ok(credentials) = existing && use_existing_creds.prompt()? {
        //TODO: Decrypt pls
        return Ok(credentials);
    }

    let username: String = Text::new("Toggl Username").prompt()?;
    //TODO: Encrypt pls
    let password = Password::new("Toggl Password").prompt()?;
    let tempo_token: String = Text::new("Tempo token").prompt()?; //TODO: Link to how to create
    let account_id: String = Text::new("Account Id").prompt()?; //TODO: Link to how to retrieve
    
    let credentials = Credentials { username, password, tempo_token, account_id };
    if Confirm::new("Stay logged in?")
        .prompt()?
    {
        let save_result = save_file(CRED_FILE, 0, &credentials);
        if save_result.is_err() {
            println!("Failed to save credentials :(");
        }
        else{
            println!("Ok, I will remember them!");
        }
    } else {
        println!("Ok, I will not save them!");
    }

    Ok(credentials)
}
