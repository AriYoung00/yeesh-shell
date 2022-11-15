use std::fs;
use std::io::Read;
use std::os::unix::fs::FileExt;

use regex::Regex;
use vfs::{FileSystem, VfsPath};

use crate::error as e;

fn get_file_matches_from_dir(search_str: &str, it: Box<dyn Iterator<Item = VfsPath>>) -> e::Result<Vec<String>> {
    Ok(it
        .filter(|x| x.filename().contains(search_str))
        .map(|x| {
            if x.is_file().is_ok_and(|b| b) {
                x.filename()
            }
            else {
                x.filename() + "/"
            }
        })
        .collect())
}

fn get_local_matches(prefix: &str, files: &impl FileSystem) -> e::Result<Vec<String>> {
    Ok(files.read_dir(prefix)?.filter(|x| x.contains(prefix)).collect())
}

fn get_path_matches(path: &str, files: &impl FileSystem) -> e::Result<Vec<String>> {
    if !path.contains("/") {
        return Err(e::YeeshError::new("No delimiters contained in path"));
    }

    let mut parts = path.split("/").collect::<Vec<&str>>();
    let prefix = parts.pop().unwrap();
    let suffix = parts.as_slice()[0..parts.len() - 1].join("/");

    Ok(files.read_dir(prefix)?.filter(|x| x.contains(&suffix)).collect())
}

pub struct TabHandler {
    idx:        usize,
    filesystem: Box<dyn FileSystem>,
}

impl TabHandler {
    pub fn new(file_system: Box<dyn FileSystem>) -> Self {
        TabHandler {
            idx:        0,
            filesystem: file_system,
        }
    }

    pub fn get_suggestion(&mut self, prefix: &String, current_dir: &String) -> Option<String> {
        let mut results = vec![];
        let ds_re = Regex::new(r"^\./").ok()?;
        let sds_re = Regex::new(r"/\./").ok()?;

        let tmp = ds_re.replace_all(&prefix, "");
        let res = sds_re.replace_all(&tmp, "/");

        if let Some((suffix, elements)) = prefix.split("/") {
            let mut path_str: String = elements.join("/");
            let root_path: VfsPath = self.filesystem.into();

            if path_str[0] == "/" {
                path_str.remove(0);
            }
            else {
                path_str = current_dir + path_str
            }

            if self.filesystem.exists(&path_str) {
                let vfs_path = root_path.join(path_str)?;
                results.append(&mut get_file_matches_from_dir(suffix, vfs_path.read_dir()?))
            }
            results.append(&mut get_file_matches_from_dir(self.filesystem))
        }
        // results.append(&mut get_file_matches_from_dir())

        // results.append(&mut get_local_matches(&res, self.filesystem).ok()?);
        // results.append(&mut get_path_matches(&res, self.filesystem).ok()?);
        // results.extend(get_local_matches(&res));
        // results.extend(get_path_matches(&res));

        if results.is_empty() {
            return None;
        }
        else {
            self.idx += 1;
        }
        if self.idx >= results.len() {
            self.idx = 0;
        }

        Some(results.remove(self.idx))
    }
}
