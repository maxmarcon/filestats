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

    for level in 0..=4 {
        dir = setup(
            &test_files,
            if level == 0 {
                None
            } else {
                Option::from(dir.as_path())
            },
        );
        if level == 0 {
            topdir = dir.clone();
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
