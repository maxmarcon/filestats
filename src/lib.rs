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
    use super::list;
    use super::SizeEntry;
    use std::fs::create_dir;
    use std::ops::Add;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, fs};

    #[test]
    fn can_list_files() {
        let mut test_files: [SizeEntry; 3] = [
            SizeEntry::new("foo", 100),
            SizeEntry::new("boo", 200),
            SizeEntry::new("goo", 300),
        ];

        let test_dir = setup(&test_files, None, None);

        let mut dir_list = list(test_dir.as_path()).unwrap();

        dir_list.sort();
        test_files.sort();

        dir_list
            .iter()
            .zip(test_files.iter())
            .for_each(|(retrieved, expected)| assert_eq!(*retrieved, *expected))
    }

    #[test]
    fn can_list_files_recursively() {
        let test_files: [SizeEntry; 3] = [
            SizeEntry::new("foo", 100),
            SizeEntry::new("boo", 200),
            SizeEntry::new("goo", 300),
        ];

        let test_dir = setup(&test_files, None, Some(1));

        let test_files_sub_dir: [SizeEntry; 3] = [
            SizeEntry::new("abc", 340),
            SizeEntry::new("def", 50),
            SizeEntry::new("ghi", 2),
        ];

        setup(&test_files_sub_dir, Some(test_dir.as_path()), None);

        let mut dir_list = list(test_dir.as_path()).unwrap();

        let mut all_test_files = Vec::new();
        all_test_files.extend_from_slice(&test_files);
        all_test_files.extend_from_slice(&test_files_sub_dir);

        dir_list.sort();
        all_test_files.sort();

        dir_list
            .iter()
            .zip(all_test_files.iter())
            .for_each(|(retrieved, expected)| assert_eq!(*retrieved, *expected))
    }

    fn setup(test_files: &[SizeEntry], dest: Option<&Path>, index: Option<u64>) -> PathBuf {
        let temp_dir = env::temp_dir();
        let parent_dir = dest.unwrap_or(temp_dir.as_path());
        let subdir = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .add(index.unwrap_or(0))
            .to_string();

        let test_dir = parent_dir.join(subdir);

        create_dir(test_dir.as_path()).expect(&format!(
            "Could not create temporary directory: {}",
            test_dir.display()
        ));

        write_test_files(test_files, test_dir.as_path());

        test_dir
    }

    fn write_test_files(files: &[SizeEntry], dest: &Path) {
        files.iter().for_each(|f| {
            fs::write(dest.join(&f.name), str::repeat("0", f.size as usize))
                .expect("failed to write test file");
        });
    }
}
