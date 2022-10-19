use std::fs;
use regex::Regex;

pub struct TabHandler {
    idx: usize
}

fn get_local_matches(prefix: &str) -> Vec<String> {
    let mut results = vec![];
    for item in fs::read_dir("./").unwrap() {
        let name = item.unwrap().file_name().into_string().unwrap();
        if name.contains(prefix) {
            results.push(name);
        }
    }
    results
}

fn get_path_matches(path: &str) -> Vec<String> {
    if !path.contains("/") {
        return vec![];
    }

    let mut results = vec![];
    let mut parts = path.split("/").collect::<Vec<&str>>();
    let prefix = parts.pop().unwrap();
    let suffix = parts.as_slice()[0..parts.len()-1].join("/");
    let prefix_path = std::path::Path::new(&prefix);

    if prefix_path.is_dir() {
        for file in fs::read_dir(prefix_path).unwrap() {
            let tmp = file.unwrap().file_name();
            let file_name = tmp.to_str().unwrap();
            if file_name.contains(&suffix) {
                results.push(file_name.to_string());
            }
        }
    }

    results
}

impl TabHandler {
    pub fn new() -> Self {
        TabHandler {
            idx: 0
        }
    }

    pub fn get_suggestion(&mut self, prefix: &String) -> Option<String> {
        let mut results = vec![];
        let ds_re = Regex::new(r"^\./").ok()?;
        let sds_re = Regex::new(r"/./").ok()?;

        let tmp = ds_re.replace_all(&prefix, "");
        let res = sds_re.replace_all(&tmp, "/");

        results.extend(get_local_matches(&res));
        results.extend(get_path_matches(&res));

        if results.is_empty() {
            return None
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
