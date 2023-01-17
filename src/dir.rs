use std::fs;
use std::io::Error as IOError;
use std::path::Path;

#[derive(Debug)]
pub struct SizeEntry {
    pub name: String,
    pub size: u64,
}

pub fn list(p: &Path) -> Result<Vec<SizeEntry>, IOError> {
    let mut vec = Vec::new();
    for dir_entry in fs::read_dir(p)? {
        let dir_entry = dir_entry?;
        let metadata = fs::metadata(dir_entry.path())?;
        let size_entry = SizeEntry {
            name: dir_entry
                .file_name()
                .into_string()
                .unwrap_or(String::from("Not valid Unicode")),
            size: metadata.len(),
        };
        vec.push(size_entry);
    }
    Ok(vec)
}
