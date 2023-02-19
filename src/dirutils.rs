#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::fs;
use std::fs::ReadDir;
use std::io::{Error as IOError, Error};
use std::iter::from_fn;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct SizeEntry {
    pub name: String,
    pub size: u64,
}

impl Clone for SizeEntry {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            size: self.size,
        }
    }
}

impl SizeEntry {
    fn new(name: &str, size: u64) -> Self {
        SizeEntry {
            name: name.to_owned(),
            size,
        }
    }
}

type Result = std::result::Result<SizeEntry, IOError>;

pub fn list(path: &Path, max_depth: Option<u32>) -> impl Iterator<Item=Result> {
    let mut paths = VecDeque::from([(path.to_owned(), 0)]);
    let mut errors: Vec<Error> = Vec::new();

    from_fn(move || -> Option<Result> {
        while let Some(error) = errors.pop() {
            return Some(Err(error));
        }

        while let Some((current_path, level)) = paths.pop_front() {
            let metadata = fs::metadata(&current_path).unwrap();

            if metadata.is_file() {
                return Some(Ok(SizeEntry::new(
                    current_path
                        .to_str()
                        .expect("Filename is not valid Unicode"),
                    metadata.len(),
                )));
            }

            let dir_entries = fs::read_dir(&current_path);
            if dir_entries.is_err() {
                return Some(Err(dir_entries.err().unwrap()));
            }

            match max_depth {
                Some(max_depth) if level > max_depth => (),
                _ => read_dir(dir_entries.unwrap(), &mut paths, &mut errors, level)
            }
        }
        None
    })
}

fn read_dir(dir_entries: ReadDir, paths: &mut VecDeque<(PathBuf, u32)>, errors: &mut Vec<Error>, level: u32) -> () {
    for dir_entry in dir_entries {
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        let metadata = fs::metadata(dir_entry.path());
        if let Err(error) = metadata {
            errors.push(error);
            continue;
        }

        paths.push_back((dir_entry.path(), level + 1));
    }
}
