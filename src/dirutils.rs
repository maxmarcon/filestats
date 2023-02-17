#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::fs;
use std::io::Error as IOError;
use std::iter::from_fn;
use std::path::Path;

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

pub fn list(path: &Path, max_depth: Option<u32>) -> impl Iterator<Item = Result> {
    let mut paths = VecDeque::from([(path.to_owned(), 0)]);

    from_fn(move || -> Option<Result> {
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

            let read_dir = fs::read_dir(&current_path);
            if let Err(error) = read_dir {
                return Some(Err(error));
            }

            for dir_entry in read_dir.unwrap() {
                let dir_entry = match dir_entry {
                    Ok(dir_entry) => dir_entry,
                    Err(error) => {
                        println!("Can't read dir entry: {}", error);
                        continue;
                    }
                };
                let metadata = fs::metadata(dir_entry.path());
                if let Err(error) = metadata {
                    println!("Can't read metadata for {:?}: {}", dir_entry, error);
                    continue;
                }

                if let Some(max_depth) = max_depth {
                    if level > max_depth {
                        continue;
                    }
                }

                paths.push_back((dir_entry.path(), level + 1));
            }
        }
        None
    })
}
