use std::path::PathBuf;
use std::{env, io};

pub fn current_dir() -> PathBuf {
    env::current_dir().expect("Unable to read current directory")
}

pub fn read_rigz_files(input: &PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::with_capacity(1);
    if input.is_dir() {
        for f in input.read_dir()? {
            files.extend(read_rigz_files(&f?.path())?);
        }
    } else if matches!(
        input.extension().map(|e| e.to_str()),
        Some(Some("rg") | Some("rigz"))
    ) {
        files.push(input.clone());
    }
    Ok(files)
}

pub fn path_to_string(path_buf: &PathBuf) -> String {
    match path_buf.to_str() {
        None => format!("Invalid Path {path_buf:?}"),
        Some(s) => s.to_string(),
    }
}
