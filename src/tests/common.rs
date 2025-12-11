use std::path::Path;

use crate::config::{load_config, Config};

pub fn root_dir() -> String {
    let dir = std::env::temp_dir().join("synxit_test_storage");
    dir.to_str().unwrap().to_string()
}
/// Sets up a test configuration and returns the Config object along with the path to the config file.
/// Creates necessary directories and writes the configuration to a TOML file.
/// Important: The storage directories are overwritten to point to temporary test directories.
pub fn setup(conf: Option<Config>) -> (Config, String) {
    let mut config = conf.unwrap_or_default();
    let root_dir = root_dir();
    config.storage.data_dir = root_dir.to_string() + "/data";
    config.storage.log_dir = root_dir.to_string() + "/logs";
    config.storage.temp_dir = root_dir.to_string() + "/temp";
    let config_file_path = root_dir.to_string() + "/test_config.toml";
    std::fs::create_dir_all(&root_dir).unwrap();
    std::fs::write(&config_file_path, toml::to_string(&config).unwrap()).unwrap();
    (
        load_config(Some(Path::new(&config_file_path))),
        config_file_path,
    )
}

pub fn cleanup() {
    let root_dir = root_dir();
    if std::path::Path::new(&root_dir).exists() {
        std::fs::remove_dir_all(&root_dir).unwrap();
    }
}
