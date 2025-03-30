use log::{error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::exit, sync::OnceLock};
use toml::Table;

use crate::{
    logger,
    storage::file::{create_dir, dir_exists, file_exists, read_file, remove_dir},
};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub network: Network,
    pub storage: Storage,
    pub auth: Auth,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Network {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Storage {
    pub data_dir: String,
    pub temp_dir: String,
    pub log_dir: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Auth {
    pub session_timeout: u64,
    pub auth_session_timeout: u64,
}

impl Default for Auth {
    fn default() -> Self {
        Auth {
            session_timeout: 60 * 60 * 24 * 7,
            auth_session_timeout: 60,
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Network {
            port: 8080,
            host: "127.0.0.1".to_string(),
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            data_dir: "/var/lib/synxit".to_string(),
            temp_dir: "/tmp/synxit".to_string(),
            log_dir: "/var/log/synxit".to_string(),
        }
    }
}

impl Storage {
    pub fn init(&self) {
        if !dir_exists(self.data_dir.as_str()) {
            if create_dir(self.data_dir.as_str()) {
                info!("Data directory created: {}", self.data_dir);
            } else {
                error!("Failed to create data directory: {}", self.data_dir);
                exit(10)
            }
        } else {
            info!("Data directory already exists: {}", self.data_dir);
        }

        if !dir_exists(self.temp_dir.as_str()) {
            if create_dir(self.temp_dir.as_str()) {
                info!("Temp directory created: {}", self.temp_dir);
            } else {
                error!("Failed to create temp directory: {}", self.temp_dir);
                exit(10)
            }
        } else {
            info!("Temp directory already exists: {}", self.temp_dir);
        }

        if !dir_exists(self.log_dir.as_str()) {
            if create_dir(self.log_dir.as_str()) {
                info!("Log directory created: {}", self.log_dir);
            } else {
                error!("Failed to create log directory: {}", self.log_dir);
                exit(10)
            }
        } else {
            info!("Log directory already exists: {}", self.log_dir);
        }

        if !dir_exists((self.data_dir.to_string() + "/users").as_str()) {
            if create_dir((self.data_dir.to_string() + "/users").as_str()) {
                info!(
                    "Users directory created: {}",
                    self.data_dir.to_string() + "/users"
                );
            } else {
                error!(
                    "Failed to create users directory: {}",
                    self.data_dir.to_string() + "/users"
                );
                exit(10)
            }
        } else {
            info!(
                "Users directory already exists: {}",
                self.data_dir.to_string() + "/users"
            );
        }
    }

    pub fn clean(&self) {
        if dir_exists(self.data_dir.as_str()) {
            if remove_dir(self.data_dir.as_str()) {
                info!("Data directory cleaned: {}", self.data_dir.as_str());
            } else {
                error!("Failed to clean data directory: {}", self.data_dir.as_str());
            }
        }
        if dir_exists(self.temp_dir.as_str()) {
            if remove_dir(self.temp_dir.as_str()) {
                info!("Temp directory cleaned: {}", self.temp_dir.as_str());
            } else {
                error!("Failed to clean temp directory: {}", self.temp_dir.as_str());
            }
        }

        if dir_exists(self.log_dir.as_str()) {
            if remove_dir(self.log_dir.as_str()) {
                info!("Log directory cleaned: {}", self.log_dir.as_str());
            } else {
                error!("Failed to clean log directory: {}", self.log_dir.as_str());
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            network: Network::default(),
            storage: Storage::default(),
            auth: Auth::default(),
        }
    }
}
pub fn load_config() {
    let mut config_default: Config = Config::default();
    let config_file: Table = read_config_file();
    let empty = Table::new();
    if config_file.contains_key("network") && config_file["network"].is_table() {
        let network = config_file["network"].as_table().unwrap_or(&empty);
        if network.contains_key("port") && network["port"].is_integer() {
            config_default.network.port =
                network["port"].as_integer().expect("Can't parse port") as u16;
        }
        if network.contains_key("host") && network["host"].is_str() {
            config_default.network.host = network["host"]
                .as_str()
                .expect("Can't parse host")
                .to_string();
        }
    }
    if config_file.contains_key("storage") {
        let storage = config_file["storage"].as_table().unwrap_or(&empty);
        if storage.contains_key("data_dir") && storage["data_dir"].is_str() {
            config_default.storage.data_dir = storage["data_dir"]
                .as_str()
                .expect("Can't parse data_dir")
                .to_string();
        }
        if storage.contains_key("temp_dir") && storage["temp_dir"].is_str() {
            config_default.storage.temp_dir = storage["temp_dir"]
                .as_str()
                .expect("Can't parse temp_dir")
                .to_string();
        }
        if storage.contains_key("log_dir") && storage["log_dir"].is_str() {
            config_default.storage.log_dir = storage["log_dir"]
                .as_str()
                .expect("Can't parse log_dir")
                .to_string();
        }
    }

    if config_file.contains_key("auth") {
        let auth = config_file["auth"].as_table().unwrap_or(&empty);
        if auth.contains_key("session_timeout") && auth["session_timeout"].is_integer() {
            config_default.auth.session_timeout = auth["session_timeout"]
                .as_integer()
                .expect("Can't parse session_timeout")
                as u64;
        }
        if auth.contains_key("auth_session_timeout") && auth["auth_session_timeout"].is_integer() {
            config_default.auth.auth_session_timeout = auth["auth_session_timeout"]
                .as_integer()
                .expect("Can't parse auth_session_timeout")
                as u64;
        }
    }

    match logger::init_logger(config_default.storage.log_dir.as_str(), LevelFilter::Debug) {
        Ok(_) => (),
        Err(_) => {
            exit(1);
        }
    }

    config_default.storage.init();
    CONFIG.get_or_init(|| config_default);
}

fn read_config_file() -> toml::Table {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if file_exists(&args[1]) {
            match read_file(Path::new(&args[1])) {
                Ok(file_content) => match toml::from_str(file_content.as_str()) {
                    Ok(config) => config,
                    Err(err) => panic!("Error parsing config file: {}", err),
                },
                Err(err) => panic!("Error reading config file: {}", err),
            }
        } else {
            toml::Table::new()
        }
    } else {
        toml::Table::new()
    }
}
