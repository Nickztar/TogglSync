use std::collections::{HashMap};

use colored::Colorize;
use inquire::{Autocomplete, CustomUserError, autocompletion::Replacement};

#[derive(Clone, Default)]
pub struct IssueCompleter {
    prev_issues: Vec<IssueKey>
}

#[derive(Clone, Default)]
struct IssueKey {
    pub key: String,
    pub desc: String,
    pub hash: String,
    pub order: u64
}

impl IssueCompleter {
    pub fn new(prev: HashMap<String, String>) -> IssueCompleter {
        let mut prev_issues = prev.iter().map(|(key, value)| {
            let (_, order_str) = key.split_once('-').unwrap_or(("", "0")); 
            let order = order_str.parse::<u64>().unwrap_or(0);
            let hash = (key.to_string() + value).to_lowercase();
            IssueKey {
                hash,
                key: key.to_string(),
                desc: value.to_string(),
                order
            }
        }).collect::<Vec<_>>();
        prev_issues.sort_by(|a, b| a.order.cmp(&b.order));
        IssueCompleter { prev_issues }
    }
}

impl Autocomplete for IssueCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {

        Ok(self.prev_issues
            .iter()
            .filter(|p| p.hash.contains(&input.to_lowercase()))
            .take(20)
            .map(|p| format!("{}: {}", p.key, p.desc.black()))
            .collect())
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        Ok(match highlighted_suggestion {
            Some(suggestion) => {
                match suggestion.split_once(':') {
                    Some((key, _)) => Replacement::Some(key.to_string()),
                    None => Replacement::None,
                }
            },
            None => Replacement::None
        })
    }
}