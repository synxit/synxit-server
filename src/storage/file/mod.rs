pub fn read_file(path: &str) -> String{
    std::fs::read_to_string(path).expect("Failed to read file")
}

pub fn write_file(path: &str, content: &str) {
    std::fs::write(path, content).expect("Failed to write file");
}

pub fn create_dir(path: &str) {
    std::fs::create_dir_all(path).expect("Failed to create directory");
}

pub fn remove_dir(path: &str) {
    std::fs::remove_dir_all(path).expect("Failed to remove directory");
}

pub fn remove_file(path: &str) {
    std::fs::remove_file(path).expect("Failed to remove file");
}

pub fn file_exists(path: &str) -> bool {
    std::fs::metadata(path).is_ok()
}

pub fn dir_exists(path: &str) -> bool {
    std::fs::metadata(path).is_ok()
}

pub fn read_dir(path: &str, recursive: bool) -> Vec<String> {
    let mut files = vec![];
    for entry in std::fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() && recursive {
            files.append(&mut read_dir(path.to_str().unwrap(), true));
        } else {
            files.push(path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    files
}
