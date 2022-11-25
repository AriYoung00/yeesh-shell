use std::cmp::Ordering;
use std::path::Path;

use filesystem::{DirEntry, FileSystem};

#[derive(PartialEq, Eq, Clone)]
pub struct Suggestion {
    pub replacement: String,
    is_prefix:       bool,
}

impl Ord for Suggestion {
    fn cmp(&self, other: &Self) -> Ordering {
        other.is_prefix.cmp(&other.is_prefix)
    }
}

impl PartialOrd for Suggestion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.is_prefix.cmp(&other.is_prefix))
    }
}

pub trait Suggester {
    fn get_suggestions(&self, prefix: &str) -> Vec<Suggestion>;
}

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

    /// Return a list of files in `path` whose name `search_str` is a substring of
    /// `search_str` should describe a path in the [FileSystem] `self.filesystem`
    fn get_suggestions(&self, path: &Path, search_str: &str) -> Vec<Suggestion> {
        self.filesystem
            .read_dir(path)
            .unwrap()
            .filter_map(|x| {
                let file_name: String = x.unwrap().file_name().to_string_lossy().into();
                if file_name.contains(search_str) {
                    Some(Suggestion {
                        replacement: path.to_string_lossy().to_string() + &file_name,
                        is_prefix:   file_name.starts_with(search_str),
                    })
                }
                else {
                    None
                }
            })
            .collect()
    }

    fn get_search_params(&self, prefix: &str) -> (Box<Path>, String) {
        let path = Path::new(prefix);
        if self.filesystem.is_dir(path) {
            (path.into(), "".to_string())
        }
        else {
            let last_slash_idx = prefix.rfind("/").unwrap_or(0);
            let prefix = &prefix[0..last_slash_idx];
            let suffix = &prefix[last_slash_idx..];

            (Path::new(prefix).into(), suffix.to_string())
        }
    }
}

impl<T: FileSystem> Suggester for FileSystemSuggester<T> {
    fn get_suggestions(&self, prefix: &str) -> Vec<Suggestion> {
        let (search_path, search_str) = self.get_search_params(prefix);
        self.get_suggestions(&search_path, &search_str)
    }
}
