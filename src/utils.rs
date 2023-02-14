
pub fn clean_description(input: &str) -> String {
    let chars: &[_] = &[':', '-'];
    input.trim().trim_matches(chars).trim().to_string()
}

pub fn clean_key(input: &str) -> String {
    match input.split_once(':') {
        Some((key, _)) => key.trim().to_string(),
        None => input.trim().to_string(),
    }
}