use std::fs;
use std::io::Error as IOError;
use std::path::Path;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct SizeEntry {
    pub name: String,
    pub size: u64,
}

impl SizeEntry {
    fn new(name: &str, size: u64) -> Self {
        SizeEntry {
            name: String::from(name),
            size,
        }
    }
}

pub fn list(path: &Path) -> Result<Vec<SizeEntry>, IOError> {
    let mut size_entries = Vec::new();
    for dir_entry in fs::read_dir(path)? {
        let dir_entry = dir_entry?;
        let metadata = fs::metadata(dir_entry.path())?;
        let size_entry = SizeEntry::new(
            dir_entry
                .file_name()
                .to_str()
                .expect("Filename is not valid Unicode"),
            metadata.len(),
        );
        size_entries.push(size_entry);
    }
    Ok(size_entries)
}

#[cfg(test)]
mod tests {
    use super::SizeEntry;
    use crate::dir::list;
    use std::fs::create_dir;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, fs};

    #[test]
    fn can_list_files() {
        let mut test_files: [SizeEntry; 3] = [
            SizeEntry::new("foo", 100),
            SizeEntry::new("boo", 200),
            SizeEntry::new("goo", 300),
        ];

        let path = setup(&test_files);

        let mut dir_list = list(path.as_path()).unwrap();

        dir_list.sort();
        test_files.sort();

        dir_list
            .iter()
            .zip(test_files.iter())
            .for_each({ |(retrieved, expected)| assert_eq!(*retrieved, *expected) })
    }

    fn setup(test_files: &[SizeEntry]) -> PathBuf {
        let temp_dir = env::temp_dir();
        let subdir = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let test_dir = temp_dir.join(subdir);

        create_dir(test_dir.as_path()).expect(&format!(
            "Could not create temporary directory: {}",
            test_dir.display()
        ));

        test_files.iter().for_each(|f| {
            fs::write(test_dir.join(&f.name), str::repeat("0", f.size as usize))
                .expect("failed to write test file");
        });

        test_dir
    }
}
