use super::list;
use super::SizeEntry;
use rand::Rng;
use std::fs::create_dir;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[test]
fn can_list_files() {
    let mut test_files = [
        SizeEntry::from(("foo", 100)),
        SizeEntry::from(("boo", 200)),
        SizeEntry::from(("goo", 300)),
    ];

    let test_dir = create_new_dir_with_files(&mut test_files, None);

    let mut dir_list = list(test_dir.as_path(), None)
        .map(|r| r.ok().unwrap())
        .collect::<Vec<_>>();

    dir_list.sort();
    test_files.sort();

    dir_list
        .iter()
        .zip(test_files.iter())
        .for_each(|(retrieved, expected)| assert_eq!(*retrieved, *expected))
}

#[test]
fn can_list_files_recursively() {
    let mut test_files = [
        SizeEntry::from(("foo", 100)),
        SizeEntry::from(("boo", 200)),
        SizeEntry::from(("goo", 300)),
    ];

    let test_dir = create_new_dir_with_files(&mut test_files, None);

    let mut test_files_sub_dir = [
        SizeEntry::from(("abc", 340)),
        SizeEntry::from(("def", 50)),
        SizeEntry::from(("ghi", 2)),
    ];

    create_new_dir_with_files(&mut test_files_sub_dir, Some(test_dir.as_path()));

    let mut dir_list = list(test_dir.as_path(), None)
        .map(|r| r.ok().unwrap())
        .collect::<Vec<_>>();

    dir_list.sort();

    let mut expected = [test_files, test_files_sub_dir].concat();
    expected.sort();

    dir_list
        .iter()
        .zip(expected.iter())
        .for_each(|(retrieved, expected)| assert_eq!(*retrieved, *expected))
}

#[test]
fn can_limit_depth() {
    let test_files = [
        SizeEntry::from(("foo", 100)),
        SizeEntry::from(("boo", 200)),
        SizeEntry::from(("goo", 300)),
    ];

    let mut dir = PathBuf::new();
    let mut topdir = PathBuf::new();

    for level in 0..=4 {
        dir = create_new_dir_with_files(
            &mut test_files.clone(),
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

    assert_eq!(
        list(topdir.as_path(), Some(3)).map(|r| r.unwrap()).count(),
        12 // 4 levels (0..=3) with 3 files each
    );
}

#[test]
fn returns_errors() {
    let mut test_files = [
        SizeEntry::from(("foo", 100)),
        SizeEntry::from(("boo", 200)),
        SizeEntry::from(("goo", 300)),
        SizeEntry::from(("Xfoo", 101)),
        SizeEntry::from(("Xboo", 202)),
        SizeEntry::from(("Xgoo", 303)),
    ];

    let topdir = create_new_dir_with_files(&mut test_files[0..4], None);

    create_new_dir_with_files(&mut test_files[4..], Some(&topdir));

    let path_with_error = topdir.join("broken_link");
    symlink("/does_not_exist", &path_with_error).unwrap();

    assert_eq!(list(topdir.as_path(), None).count(), 7);

    let error = list(topdir.as_path(), None).find(|r| r.is_err());
    assert!(error.is_some());

    assert_eq!(error.unwrap().err().unwrap().path, path_with_error);
}

#[test]
fn accept_file_as_input_path() {
    let mut test_file = [SizeEntry::from(("foo", 200))];

    create_new_dir_with_files(&mut test_file, None);

    let size_entries = list(test_file[0].path.as_path(), None)
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();

    assert_eq!(size_entries, test_file)
}

/// Creates `test_files` in a new dir with a randomly generated name
/// place dir in `dest` (if provided) or in a temporary system folder
fn create_new_dir_with_files(test_files: &mut [SizeEntry], dest: Option<&Path>) -> PathBuf {
    let mut rng = rand::thread_rng();

    let temp_dir = env::temp_dir();
    let parent_dir = dest.unwrap_or(temp_dir.as_path());
    let subdir: u32 = rng.gen();
    let test_dir = parent_dir.join(format!("{}", subdir));

    create_dir(test_dir.as_path()).expect(&format!(
        "Could not create temporary directory: {}",
        test_dir.display()
    ));

    test_files.iter_mut().for_each(|f| {
        f.path = test_dir.join(&f.path);
        fs::write(&f.path, str::repeat("0", f.size as usize)).expect("failed to write test file");
    });

    test_dir
}
