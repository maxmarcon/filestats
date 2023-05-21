#[cfg(test)]
mod tests;

use std::collections::vec_deque::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs;
use std::iter::from_fn;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct FileSize {
    pub path: PathBuf,
    pub size: u64,
}

type Result = std::result::Result<FileSize, Error>;

enum PathBit {
    Result(Result),
    Dir((PathBuf, u32)),
}

impl FileSize {
    fn new(path: PathBuf, size: u64) -> Self {
        Self { path, size }
    }
}

impl From<(&str, u64)> for FileSize {
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

const RESULT_BUFFER_SIZE: usize = 1000;

pub fn traverse(path: &Path, max_depth: Option<u32>) -> impl Iterator<Item = Result> {
    let mut result_queue = VecDeque::with_capacity(RESULT_BUFFER_SIZE);
    let mut dir_queue = VecDeque::new();
    match read_path(path, 0) {
        Some(PathBit::Dir((path, _))) => dir_queue.push_back((path, 0)),
        Some(PathBit::Result(result)) => result_queue.push_back(result),
        None => (),
    }

    from_fn(move || -> Option<Result> {
        if result_queue.is_empty() && !dir_queue.is_empty() {
            let path_bits = dir_queue
                .iter()
                .flat_map(|(path, depth)| read_dir(path, *depth))
                .collect::<Vec<PathBit>>();
            dir_queue.clear();

            path_bits.into_iter().for_each(|path_bit| match path_bit {
                PathBit::Result(result) => result_queue.push_back(result),
                PathBit::Dir((path, depth)) => {
                    if max_depth.map_or(true, |max_depth| depth <= max_depth) {
                        dir_queue.push_back((path, depth))
                    }
                }
            });
        }

        result_queue.pop_front()
    })
}

fn read_dir(dir_path: &Path, depth: u32) -> Vec<PathBit> {
    let dir_entries = fs::read_dir(dir_path);
    if dir_entries.is_err() {
        return vec![PathBit::Result(Err(Error::new(
            dir_path.to_owned(),
            dir_entries.err().unwrap(),
        )))];
    }

    dir_entries
        .unwrap()
        .into_iter()
        .map_while(|dir_entry| match dir_entry {
            Ok(dir_entry) => read_path(&dir_entry.path(), depth),
            Err(error) => Some(PathBit::Result(Err(Error::new(dir_path.to_owned(), error)))),
        })
        .collect()
}

fn read_path(path: &Path, depth: u32) -> Option<PathBit> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => Some(PathBit::Result(Ok(FileSize::new(
            path.to_owned(),
            metadata.len(),
        )))),
        Ok(metadata) if metadata.is_dir() => Some(PathBit::Dir((path.to_owned(), depth + 1))),
        Ok(_) => None,
        Err(error) => Some(PathBit::Result(Err(Error::new(path.to_owned(), error)))),
    }
}
