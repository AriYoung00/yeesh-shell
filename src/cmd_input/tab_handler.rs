use filesystem::FileSystem;

use crate::cmd_input::suggester::{FileSystemSuggester, Suggester, Suggestion};

pub struct TabHandler {
    suggesters:  Vec<Box<dyn Suggester>>,
    last_input:  String,
    cached_iter: Box<dyn Iterator<Item = Suggestion>>,
}

impl TabHandler {
    pub fn new<T: FileSystem + 'static>(fs: T) -> Self {
        let tmp = vec![];
        TabHandler {
            suggesters:  vec![Box::new(FileSystemSuggester::new(fs))],
            last_input:  "poqjkfdpqokjpioejq".to_string(),
            cached_iter: Box::new(tmp.into_iter().cycle()),
        }
    }

    #[cfg(test)]
    pub fn set_suggesters(&mut self, suggesters: Vec<Box<dyn Suggester>>) {
        self.suggesters = suggesters;
    }

    pub fn get_suggesters(&self) -> &Vec<Box<dyn Suggester>> {
        &self.suggesters
    }

    /// Returns an `Option<String>` representing the value that the current token should be
    /// replaced with, or `None` if there are no suggestions.
    ///
    pub fn get_suggestion(&mut self, prefix: &String) -> Option<String> {
        if prefix != &self.last_input {
            let mut suggestions = self.suggesters.iter_mut().fold(vec![], |mut acc, s| {
                acc.append(&mut s.get_suggestions(prefix));
                acc
            });
            suggestions.sort();
            self.cached_iter = Box::new(suggestions.into_iter().cycle());
            self.last_input = prefix.clone();
        }

        self.cached_iter.next().map(|suggestion| suggestion.replacement)
    }
}
