use std::collections::HashMap;

use colored::Colorize;
use inquire::{Autocomplete, CustomUserError, autocompletion::Replacement};

#[derive(Clone, Default)]
pub struct IssueCompleter {
    prev_issues: HashMap<String, String>
}

impl IssueCompleter {
    pub fn new(prev: HashMap<String, String>) -> IssueCompleter {
        IssueCompleter { prev_issues: prev }
    }
}

impl Autocomplete for IssueCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {

        Ok(self.prev_issues
            .iter()
            .filter(|p| p.0.to_lowercase().contains(&input))
            .take(5)
            .map(|p| format!("{}: {}", p.0.blue(), p.1.bright_black()))
            .collect())
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        Ok(match highlighted_suggestion {
            Some(suggestion) => {
                match suggestion.split_once(' ') {
                    Some((key, _)) => Replacement::Some(key.to_string()),
                    None => Replacement::Some(suggestion),
                }
            },
            None => Replacement::None
        })
    }
}