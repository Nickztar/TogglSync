use std::collections::{HashSet};

use savefile::{load_file, save_file};
const KEY_FILE: &str = "issue_keys.bin";

pub fn retreive_keys() -> anyhow::Result<HashSet<String>> {
    let existing = load_file::<HashSet<String>, _>(KEY_FILE, 0);

    if let Ok(keys) = existing {
        return Ok(keys);
    }
    else {
        return Ok(HashSet::new());
    }
}

pub fn store_keys(keys: HashSet<String>) -> anyhow::Result<()> {
    let _ = save_file(KEY_FILE, 0, &keys)?;
    Ok(())
}

