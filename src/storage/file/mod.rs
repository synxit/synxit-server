use std::fs;
use std::io;
use std::path::Path;

/// Reads the entire contents of a file into a string.
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

/// Writes a string to a file, returning true on success and false on failure.
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> bool {
    match fs::write(path, content) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Creates a directory and all necessary parent directories, returning true on success and false on failure.
pub fn create_dir<P: AsRef<Path>>(path: P) -> bool {
    fs::create_dir_all(path).is_ok()
}

/// Removes a directory and all its contents, returning true on success and false on failure.
pub fn remove_dir<P: AsRef<Path>>(path: P) -> bool {
    match fs::remove_dir_all(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Removes a file, returning true on success and false on failure.
pub fn remove_file<P: AsRef<Path>>(path: P) -> bool {
    match fs::remove_file(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Checks if a file exists at the given path.
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Checks if a directory exists at the given path.
pub fn dir_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Reads the contents of a directory, returning a vector of file and directory names.
pub fn read_dir<P: AsRef<Path>>(path: P, recursive: bool) -> io::Result<Vec<String>> {
    let mut files = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && recursive {
            files.append(&mut read_dir(path, true)?);
        } else {
            files.push(path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    Ok(files)
}

/// Calculates the total size of a folder, including all its files and subdirectories.
pub fn get_folder_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    let mut size = get_file_size(&path)?;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            size += get_folder_size(&path)?;
        } else {
            size += get_file_size(path)?
        }
    }
    Ok(size)
}

/// Gets the size of a file in bytes.
pub fn get_file_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    Ok(path.as_ref().metadata()?.len())
}
