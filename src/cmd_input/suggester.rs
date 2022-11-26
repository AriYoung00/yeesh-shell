use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::path::{Path, PathBuf};

use filesystem::{DirEntry, FileSystem};
use itertools::Itertools;

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

    fn get_suggestion_from_file(&self, file: &impl DirEntry, path: &Path, search_str: &str) -> Option<Suggestion> {
        let file_name: String = file.file_name().to_string_lossy().into();
        if file_name.contains(search_str) {
            let s_type = SuggestionType::from_pathbuf(&file.path(), &self.filesystem);
            let replacement_suffix = if s_type == Directory {
                file_name + "/"
            }
            else {
                file_name
            };

            Some(Suggestion {
                replacement: path.to_string_lossy().to_string() + &replacement_suffix,
                is_prefix: replacement_suffix.starts_with(search_str),
                s_type,
            })
        }
        else {
            None
        }
    }

    /// Return a list of files in `path` whose name `search_str` is a substring of
    /// `search_str` should describe a path in the [FileSystem] `self.filesystem`
    fn _get_suggestions(&self, path: &Path, search_str: &str) -> Option<Vec<Suggestion>> {
        Some(
            self.filesystem
                .read_dir(path)
                .ok()?
                .filter_map(|x| self.get_suggestion_from_file(&x.unwrap(), path, search_str))
                .sorted()
                .collect(),
        )
    }

    pub(super) fn get_search_params(&self, prefix: &str) -> (Box<Path>, String) {
        let path = Path::new(prefix);
        if self.filesystem.is_dir(path) && prefix.ends_with('/') {
            (path.into(), "".to_string())
        }
        else {
            let last_slash_idx = match prefix.rfind('/') {
                Some(i) => i + 1,
                None => 0,
            };
            let new_prefix = &prefix[..last_slash_idx];
            let suffix = &prefix[last_slash_idx..];

            (Path::new(new_prefix).into(), suffix.to_string())
        }
    }
}

impl<T: FileSystem> Suggester for FileSystemSuggester<T> {
    fn get_suggestions(&mut self, prefix: &str) -> Vec<Suggestion> {
        let (search_path, search_str) = self.get_search_params(prefix);
        self._get_suggestions(&search_path, &search_str).unwrap_or_default()
    }

    #[cfg(test)]
    fn get_get_suggestion_count(&self) -> usize {
        0
    }
}
