use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::path::{Path, PathBuf};

use filesystem::{DirEntry, FileSystem};
use itertools::Itertools;
use log::{debug, error, info, trace};

use crate::cmd_input::suggester::SuggestionType::{Directory, File};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SuggestionType {
    Directory,
    File,
    /// an executable in $PATH
    PathExecutable,
}

impl SuggestionType {
    pub fn from_pathbuf(path: &PathBuf, filesystem: &impl FileSystem) -> Self {
        if filesystem.is_dir(path) {
            Directory
        }
        else {
            File
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Suggestion {
    pub replacement:      String,
    pub(super) is_prefix: bool,
    pub s_type:           SuggestionType,
}

impl Ord for Suggestion {
    fn cmp(&self, other: &Self) -> Ordering {
        let res = self.is_prefix.cmp(&other.is_prefix).reverse();
        if res == Equal {
            self.replacement.cmp(&other.replacement)
        }
        else {
            res
        }
    }
}

impl PartialOrd for Suggestion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub trait Suggester {
    fn get_suggestions(&mut self, prefix: &str) -> Vec<Suggestion>;

    #[cfg(test)]
    fn get_get_suggestion_count(&self) -> usize;
}

#[derive(Clone)]
pub struct FileSystemSuggester<T>
where
    T: FileSystem,
{
    filesystem: T,
}

impl<T: FileSystem> FileSystemSuggester<T> {
    pub fn new(filesystem: T) -> Self {
        FileSystemSuggester { filesystem }
    }

    fn get_suggestion_from_file(&self, file: &impl DirEntry, path: &str, search_str: &str) -> Option<Suggestion> {
        trace!(
            "Getting suggestion from file '{:?}', path '{}', search_str '{}'",
            file.file_name().to_string_lossy(),
            path,
            search_str
        );
        let file_name: String = file.file_name().to_string_lossy().into();
        if file_name.contains(search_str) {
            trace!("File name matches");
            let s_type = SuggestionType::from_pathbuf(&file.path(), &self.filesystem);
            let replacement_suffix = if s_type == Directory {
                file_name + "/"
            }
            else {
                file_name
            };

            Some(Suggestion {
                replacement: path.to_string() + &replacement_suffix,
                is_prefix: replacement_suffix.starts_with(search_str),
                s_type,
            })
        }
        else {
            trace!("File name does not match");
            None
        }
    }

    /// Return a list of files in `path` whose name `search_str` is a substring of
    /// `search_str` should describe a path in the [FileSystem] `self.filesystem`
    fn _get_suggestions(&self, path: &str, search_str: &str) -> io::Result<Vec<Suggestion>> {
        let search_path = if path.is_empty() || path == "./" {
            self.filesystem.current_dir().unwrap()
        }
        else {
            path.into()
        };

        Ok(self
            .filesystem
            .read_dir(search_path)?
            .filter_map(|x| self.get_suggestion_from_file(&x.unwrap(), path, search_str))
            .sorted()
            .collect())
    }

    pub(super) fn get_search_params(&self, prefix: &str) -> (String, String) {
        let path = Path::new(prefix);
        if self.filesystem.is_dir(path) && prefix.ends_with('/') {
            (path.to_string_lossy().to_string(), "".to_string())
        }
        else {
            let last_slash_idx = match prefix.rfind('/') {
                Some(i) => i + 1,
                None => 0,
            };
            let new_prefix = &prefix[..last_slash_idx];
            let suffix = &prefix[last_slash_idx..];

            (new_prefix.to_string(), suffix.to_string())
        }
    }
}

impl<T: FileSystem> Suggester for FileSystemSuggester<T> {
    fn get_suggestions(&mut self, prefix: &str) -> Vec<Suggestion> {
        debug!("FileSystemSuggester - Getting suggestions for prefix '{}'", prefix);
        let (search_path, search_str) = self.get_search_params(prefix);
        trace!(
            "Got search params, path: '{:?}', search_str: '{}'",
            search_path,
            search_str
        );
        let suggestions_res = self._get_suggestions(&search_path, &search_str);

        if let Err(e) = &suggestions_res {
            error!(
                "Unable to get suggestions from path '{}', search_str '{}', reason: '{}'",
                search_path, search_str, e
            );
        }
        suggestions_res.unwrap_or_default()
    }

    #[cfg(test)]
    fn get_get_suggestion_count(&self) -> usize {
        0
    }
}
