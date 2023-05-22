use rayon::iter::ParallelDrainRange;
use rayon::iter::ParallelIterator;
use std::cmp::min;
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

// max number of paths that will be traversed in parallel at any time
const MAX_PARALLEL_PATHS: u32 = 1_000;

pub fn visit(
    path: &Path,
    max_depth: Option<u32>,
    max_parallel: Option<u32>,
) -> impl Iterator<Item = Result> {
    let max_parallel = max_parallel.unwrap_or(MAX_PARALLEL_PATHS) as usize;
    let mut result_queue = VecDeque::new();
    let mut dir_queue = VecDeque::new();
    match read_path(path, 0) {
        Some(PathBit::Dir((path, _))) => dir_queue.push_back((path, 0)),
        Some(PathBit::Result(result)) => result_queue.push_back(result),
        None => (),
    }

    from_fn(move || -> Option<Result> {
        while result_queue.is_empty() && !dir_queue.is_empty() {
            let path_bits = dir_queue
                .par_drain(..(min(max_parallel, dir_queue.len())))
                .flat_map(|(path, depth)| read_dir(&path, depth))
                .collect::<Vec<PathBit>>();

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

#[cfg(test)]
mod tests {
    use super::visit;
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

        let mut dir_list = visit(test_dir.as_path(), None, Some(1))
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

        let mut dir_list = visit(test_dir.as_path(), None, Some(1))
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
            visit(topdir.as_path(), Some(3), Some(1))
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

        let error = visit(temp_dir.as_path(), None, Some(1)).next().unwrap();
        assert!(error.is_err());

        assert_eq!(error.err().unwrap().path, path_with_error);
    }

    #[test]
    fn accepts_file_as_input_path() {
        let mut test_file = [FileSize::from(("foo", 200))];

        create_new_dir_with_files(&mut test_file, None);

        let size_entries = visit(test_file[0].path.as_path(), None, Some(1))
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();

        assert_eq!(size_entries, test_file)
    }

    #[test]
    fn accepts_nonexistent_paths() {
        let mut rng = rand::thread_rng();

        let path = PathBuf::from(format!("{}", rng.gen::<u32>()));

        let result = visit(path.as_path(), None, Some(1)).next().unwrap();

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
            fs::write(&f.path, str::repeat("0", f.size as usize))
                .expect("failed to write test file");
        });

        test_dir
    }
}
