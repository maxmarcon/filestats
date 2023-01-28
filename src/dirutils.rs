#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::fs;
use std::io::Error as IOError;
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

pub fn list(path: &Path, max_depth: Option<u32>) -> Result<Vec<SizeEntry>, IOError> {
    let mut size_entries = Vec::new();

    let mut paths = VecDeque::from([(path.to_owned(), 0)]);

    while paths.len() > 0 {
        let (current_path, level) = paths.pop_front().unwrap();
        for dir_entry in fs::read_dir(&current_path)? {
            let dir_entry = dir_entry?;
            let metadata = fs::metadata(dir_entry.path())?;

            if metadata.file_type().is_dir() {
                match max_depth {
                    Some(max_depth) if level >= max_depth => (),
                    _ => paths.push_back((dir_entry.path(), level + 1)),
                }
            } else if metadata.file_type().is_file() {
                let size_entry = SizeEntry::new(
                    dir_entry
                        .file_name()
                        .to_str()
                        .expect("Filename is not valid Unicode"),
                    metadata.len(),
                );
                size_entries.push(size_entry);
            }
        }
    }
    Ok(size_entries)
}
