use std::collections::HashMap;

use colored::Colorize;
use inquire::{MultiSelect, Text};
use savefile::{load_file, save_file};
const KEY_FILE: &str = "issue_history.bin";

pub fn retreive_keys() -> anyhow::Result<HashMap<String, String>> {
    let existing = load_file::<HashMap<String, String>, _>(KEY_FILE, 0);

    if let Ok(keys) = existing {
        return Ok(keys);
    } else {
        return Ok(HashMap::new());
    }
}

pub fn store_keys(keys: HashMap<String, String>) -> anyhow::Result<()> {
    let _ = save_file(KEY_FILE, 0, &keys)?;
    Ok(())
}

pub fn add_key() -> anyhow::Result<()> {
    let mut keys = retreive_keys()?;
    println!("Current keys:");
    for (key, value) in keys.iter() {
        println!("{}", format_key(key, value));
    }
    let possible_key = Text::new("New key?").prompt_skippable()?;
    match possible_key {
        Some(key) => match Text::new("New description?").prompt_skippable()? {
            Some(desc) => {
                match keys.insert(key.to_string(), desc.to_string()) {
                    Some(_) => println!("{}: {} -> Added", key, desc),
                    None => println!("{}: {} -> ERORR - Present", key, desc),
                }
                add_key()?;
            }
            None => {}
        },
        None => {}
    }
    Ok(())
}

pub fn filter_keys() -> anyhow::Result<()> {
    let keys = retreive_keys()?;
    let options = keys
        .iter()
        .map(|(key, value)| format_key(key, value))
        .collect::<Vec<String>>();
    let selected = MultiSelect::new("Select the keys to remove:", options).prompt()?;

    let new_keys = keys
        .into_iter()
        .filter(|(key, value)| !selected.contains(&format_key(key, value)))
        .collect::<HashMap<String, String>>();

    store_keys(new_keys)
}

fn format_key(key: &str, value: &str) -> String {
    format!("{}: {}", key.blue().underline(), value.black())
}
