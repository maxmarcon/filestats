#[cfg(test)]
mod tests;

use rayon::ThreadPoolBuilder;
use std::collections::vec_deque::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs;
use std::iter::from_fn;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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

const RESULT_BUFFER_SIZE: usize = 1000;

pub fn list(path: &Path, max_depth: Option<u32>) -> impl Iterator<Item = Result> {
    let paths = Arc::new(Mutex::new(VecDeque::from([(path.to_owned(), 0)])));
    let thread_pool = ThreadPoolBuilder::new().build().unwrap();
    let result_buffer = Arc::new(Mutex::new(VecDeque::with_capacity(RESULT_BUFFER_SIZE)));

    from_fn(move || -> Option<Result> {
        loop {
            if let Some(result) = result_buffer.lock().unwrap().pop_front() {
                return Some(result);
            }

            if paths.lock().unwrap().is_empty() {
                return None;
            }

            thread_pool.scope(|s| {
                while let Some((current_path, level)) = paths.lock().unwrap().pop_front() {
                    let metadata = match fs::symlink_metadata(&current_path) {
                        Ok(metadata) => metadata,
                        Err(error) => {
                            result_buffer
                                .lock()
                                .unwrap()
                                .push_back(Err(Error::new(current_path, error)));
                            continue;
                        }
                    };

                    if metadata.is_file() {
                        result_buffer
                            .lock()
                            .unwrap()
                            .push_back(Ok(SizeEntry::new(current_path.clone(), metadata.len())));
                    }

                    if metadata.is_dir() {
                        match max_depth {
                            Some(max_depth) if level > max_depth => (),
                            _ => {
                                let paths = Arc::clone(&paths);
                                let result_buffer = Arc::clone(&result_buffer);
                                s.spawn(move |_| {
                                    read_dir(current_path, paths, result_buffer, level)
                                });
                            }
                        }
                    }

                    if result_buffer.lock().unwrap().len() >= RESULT_BUFFER_SIZE {
                        return;
                    }
                }
            });
        }
    })
}

fn read_dir(
    dir_path: PathBuf,
    paths: Arc<Mutex<VecDeque<(PathBuf, u32)>>>,
    result_buffer: Arc<Mutex<VecDeque<Result>>>,
    level: u32,
) -> () {
    let dir_entries = fs::read_dir(&dir_path);
    if dir_entries.is_err() {
        result_buffer
            .lock()
            .unwrap()
            .push_back(Err(Error::new(dir_path, dir_entries.err().unwrap())));
        return;
    }

    for dir_entry in dir_entries.unwrap() {
        match dir_entry {
            Ok(dir_entry) => paths
                .lock()
                .unwrap()
                .push_back((dir_entry.path(), level + 1)),
            Err(error) => {
                result_buffer
                    .lock()
                    .unwrap()
                    .push_back(Err(Error::new(dir_path.clone(), error)));
                continue;
            }
        };
    }
}
