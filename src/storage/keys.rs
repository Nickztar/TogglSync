use std::collections::HashMap;

use savefile::{load_file, save_file};
const KEY_FILE: &str = "issue_history.bin";

pub fn retreive_keys() -> anyhow::Result<HashMap<String, String>> {
    let existing = load_file::<HashMap<String, String>, _>(KEY_FILE, 0);

    if let Ok(keys) = existing {
        return Ok(keys);
    }
    else {
        return Ok(HashMap::new());
    }
}

pub fn store_keys(keys: HashMap<String, String>) -> anyhow::Result<()> {
    let _ = save_file(KEY_FILE, 0, &keys)?;
    Ok(())
}

