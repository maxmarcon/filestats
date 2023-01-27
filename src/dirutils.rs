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

pub fn list(path: &Path, _max_depth: Option<u32>) -> Result<Vec<SizeEntry>, IOError> {
    let mut size_entries = Vec::new();

    let mut paths = VecDeque::from([path.to_owned()]);
    let mut current_path;
    while paths.len() > 0 {
        current_path = paths.pop_front().unwrap();
        for dir_entry in fs::read_dir(&current_path)? {
            let dir_entry = dir_entry?;
            let metadata = fs::metadata(dir_entry.path())?;
            if metadata.file_type().is_dir() {
                paths.push_back(dir_entry.path());
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

#[cfg(test)]
mod tests {
    use super::list;
    use super::SizeEntry;
    use rand::Rng;
    use std::fs::create_dir;
    use std::path::{Path, PathBuf};
    use std::{env, fs};

    #[test]
    fn can_list_files() {
        let mut test_files = vec![
            SizeEntry::new("foo", 100),
            SizeEntry::new("boo", 200),
            SizeEntry::new("goo", 300),
        ];

        let test_dir = setup(&test_files, None);

        let mut dir_list = list(test_dir.as_path(), None).unwrap();

        dir_list.sort();
        test_files.sort();

        dir_list
            .iter()
            .zip(test_files.iter())
            .for_each(|(retrieved, expected)| assert_eq!(*retrieved, *expected))
    }

    #[test]
    fn can_list_files_recursively() {
        let mut test_files = vec![
            SizeEntry::new("foo", 100),
            SizeEntry::new("boo", 200),
            SizeEntry::new("goo", 300),
        ];

        let test_dir = setup(&test_files, None);

        let mut test_files_sub_dir = vec![
            SizeEntry::new("abc", 340),
            SizeEntry::new("def", 50),
            SizeEntry::new("ghi", 2),
        ];

        setup(&test_files_sub_dir, Some(test_dir.as_path()));

        let mut dir_list = list(test_dir.as_path(), None).unwrap();

        test_files.append(&mut test_files_sub_dir);

        dir_list.sort();
        test_files.sort();

        dir_list
            .iter()
            .zip(test_files.iter())
            .for_each(|(retrieved, expected)| assert_eq!(*retrieved, *expected))
    }

    #[test]
    fn can_limit_depth() {
        let test_files = vec![
            SizeEntry::new("foo", 100),
            SizeEntry::new("boo", 200),
            SizeEntry::new("goo", 300),
        ];

        let mut dir = PathBuf::new();
        let mut topdir = PathBuf::new();

        for level in 1..=10 {
            dir = setup(&test_files, if level == 1 { None } else { Option::from(dir.as_path()) });
            if level == 1 {
                topdir = dir.to_owned();
            }
        }

        assert_eq!(list(topdir.as_path(), Some(3)).unwrap().len(), 12);
    }

    fn setup(test_files: &[SizeEntry], dest: Option<&Path>) -> PathBuf {
        let mut rng = rand::thread_rng();

        let temp_dir = env::temp_dir();
        let parent_dir = dest.unwrap_or(temp_dir.as_path());
        let subdir: u32 = rng.gen();
        let test_dir = parent_dir.join(format!("{}", subdir));

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
