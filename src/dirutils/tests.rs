use super::traverse;
use super::FileSize;
use rand::Rng;
use std::fs::create_dir;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[test]
fn can_list_files() {
    let mut test_files = [
        FileSize::from(("foo", 100)),
        FileSize::from(("boo", 200)),
        FileSize::from(("goo", 300)),
    ];

    let test_dir = create_new_dir_with_files(&mut test_files, None);

    let mut dir_list = traverse(test_dir.as_path(), None)
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
        FileSize::from(("foo", 100)),
        FileSize::from(("boo", 200)),
        FileSize::from(("goo", 300)),
    ];

    let test_dir = create_new_dir_with_files(&mut test_files, None);

    let mut test_files_sub_dir = [
        FileSize::from(("abc", 340)),
        FileSize::from(("def", 50)),
        FileSize::from(("ghi", 2)),
    ];

    create_new_dir_with_files(&mut test_files_sub_dir, Some(test_dir.as_path()));

    let mut dir_list = traverse(test_dir.as_path(), None)
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
        FileSize::from(("foo", 100)),
        FileSize::from(("boo", 200)),
        FileSize::from(("goo", 300)),
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
        traverse(topdir.as_path(), Some(3))
            .map(|r| r.unwrap())
            .count(),
        12 // 4 levels (0..=3) with 3 files each
    );
}

#[test]
#[ignore]
fn returns_errors() {
    let temp_dir = env::temp_dir();
    let path_with_error = temp_dir.join("broken_link");
    symlink("/does_not_exist", &path_with_error).unwrap();

    let error = traverse(temp_dir.as_path(), None).next().unwrap();
    assert!(error.is_err());

    assert_eq!(error.err().unwrap().path, path_with_error);
}

#[test]
fn accepts_file_as_input_path() {
    let mut test_file = [FileSize::from(("foo", 200))];

    create_new_dir_with_files(&mut test_file, None);

    let size_entries = traverse(test_file[0].path.as_path(), None)
        .map(|r| r.unwrap())
        .collect::<Vec<_>>();

    assert_eq!(size_entries, test_file)
}

#[test]
fn accepts_nonexistent_paths() {
    let mut rng = rand::thread_rng();

    let path = PathBuf::from(format!("{}", rng.gen::<u32>()));

    let result = traverse(path.as_path(), None).next().unwrap();

    assert!(result.is_err());

    assert_eq!(result.err().unwrap().path, path);
}

/// Creates `test_files` in a new dir with a randomly generated name
/// place dir in `dest` (if provided) or in a temporary system folder
fn create_new_dir_with_files(test_files: &mut [FileSize], dest: Option<&Path>) -> PathBuf {
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
