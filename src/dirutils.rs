#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs;
use std::iter::from_fn;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct SizeEntry {
    pub path: PathBuf,
    pub size: u64,
}

impl SizeEntry {
    fn new(path: PathBuf, size: u64) -> Self {
        Self { path, size }
    }
}

impl From<(&str, u64)> for SizeEntry {
    fn from((path, size): (&str, u64)) -> Self {
        Self::new(PathBuf::from(path), size)
    }
}

#[derive(Debug)]
pub struct Error {
    path: PathBuf,
    io_error: std::io::Error,
}

impl Error {
    fn new(path: PathBuf, io_error: std::io::Error) -> Self {
        Self { path, io_error }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error while reading path {:?}: {}",
            self.path, self.io_error
        )
    }
}

type Result = std::result::Result<SizeEntry, Error>;

pub fn list(path: &Path, max_depth: Option<u32>) -> impl Iterator<Item = Result> {
    let paths = Arc::new(Mutex::new(VecDeque::from([(path.to_owned(), 0)])));
    let errors = Arc::new(Mutex::new(Vec::new()));
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    from_fn(move || -> Option<Result> {
        loop {
            if handles.is_empty() && paths.lock().unwrap().is_empty() {
                return None;
            }

            while let Some(handle) = handles.pop() {
                handle.join().unwrap();
            }

            while let Some(error) = errors.lock().unwrap().pop() {
                return Some(Err(error));
            }

            while let Some((current_path, level)) = paths.lock().unwrap().pop_front() {
                let metadata = match fs::symlink_metadata(&current_path) {
                    Ok(metadata) => metadata,
                    Err(error) => return Some(Err(Error::new(current_path, error))),
                };

                if metadata.is_file() {
                    return Some(Ok(SizeEntry::new(current_path, metadata.len())));
                }

                if metadata.is_dir() {
                    match max_depth {
                        Some(max_depth) if level > max_depth => (),
                        _ => {
                            let paths = paths.clone();
                            let errors = errors.clone();
                            let handle =
                                spawn(move || read_dir(current_path, paths, errors, &level));

                            handles.push(handle);
                        }
                    }
                }
            }
        }
    })
}

fn read_dir(
    dir_path: PathBuf,
    paths: Arc<Mutex<VecDeque<(PathBuf, u32)>>>,
    errors: Arc<Mutex<Vec<Error>>>,
    level: &u32,
) -> () {
    let dir_entries = fs::read_dir(&dir_path);
    if dir_entries.is_err() {
        errors
            .lock()
            .unwrap()
            .push(Error::new(dir_path, dir_entries.err().unwrap()));
        return;
    }

    for dir_entry in dir_entries.unwrap() {
        let dir_entry = match dir_entry {
            Ok(dir_entry) => dir_entry,
            Err(error) => {
                errors
                    .lock()
                    .unwrap()
                    .push(Error::new(dir_path.clone(), error));
                continue;
            }
        };
        let metadata = fs::metadata(dir_entry.path());
        if let Err(error) = metadata {
            errors
                .lock()
                .unwrap()
                .push(Error::new(dir_entry.path(), error));
            continue;
        }

        paths
            .lock()
            .unwrap()
            .push_back((dir_entry.path(), level + 1));
    }
}
