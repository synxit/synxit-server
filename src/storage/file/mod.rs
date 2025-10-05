use std::fs;
use std::io;
use std::path::Path;

pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> bool {
    match fs::write(path, content) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn create_dir<P: AsRef<Path>>(path: P) -> bool {
    match fs::create_dir_all(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn remove_dir<P: AsRef<Path>>(path: P) -> bool {
    match fs::remove_dir_all(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> bool {
    match fs::remove_file(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

pub fn dir_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

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

pub fn get_file_size<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    Ok(path.as_ref().metadata()?.len())
}
