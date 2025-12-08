use log::{error, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::exit, sync::OnceLock};
use toml::Table;

use crate::{
    logger,
    storage::file::{create_dir, dir_exists, read_file_to_string, remove_dir},
};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Config {
    pub network: Network,
    pub storage: Storage,
    pub auth: Auth,
    pub tiers: Vec<Tier>,
    pub federation: Federation,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Federation {
    pub enabled: bool,
    pub blacklist: Blacklist,
    pub whitelist: Whitelist,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Blacklist {
    pub enabled: bool,
    pub hosts: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Whitelist {
    pub enabled: bool,
    pub hosts: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Network {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tier {
    pub id: String,
    pub name: String,
    pub description: String,
    pub quota: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Storage {
    pub data_dir: String,
    pub temp_dir: String,
    pub log_dir: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Auth {
    pub session_timeout: u64,
    pub auth_session_timeout: u64,
    pub registration_enabled: bool,
}

impl Default for Auth {
    fn default() -> Self {
        Auth {
            session_timeout: 60 * 60 * 24 * 7,
            auth_session_timeout: 60,
            registration_enabled: true,
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Network {
            port: 8044,
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

impl Default for Federation {
    fn default() -> Self {
        Federation {
            enabled: true,
            blacklist: Blacklist::default(),
            whitelist: Whitelist::default(),
        }
    }
}

impl Storage {
    /// Initialize storage directories, creating them if they don't exist.
    pub fn init(&self) {
        let directories = [
            &self.temp_dir,
            &self.data_dir,
            &(self.data_dir.clone() + "/users"),
            &self.log_dir,
        ];

        self.delete_temp();

        for dir in directories {
            if !dir_exists(dir) {
                if create_dir(dir) {
                    info!("Directory created: {}", dir);
                } else {
                    error!("Failed to create directory: {}", dir);
                    exit(10);
                }
            } else {
                info!("Directory already exists: {}", dir);
            }
        }
    }

    /// Delete temp directory
    pub fn delete_temp(&self) {
        if dir_exists(&self.temp_dir) {
            if remove_dir(&self.temp_dir) {
                info!("Temp directory deleted: {}", self.temp_dir);
            } else {
                error!("Failed to delete temp directory: {}", self.temp_dir);
            }
        }
    }

    /// Clean up storage directories by removing them.
    pub fn clean(&self) {
        let directories = [&self.data_dir, &self.temp_dir, &self.log_dir];

        for dir in directories {
            if dir_exists(dir) {
                if remove_dir(dir) {
                    info!("Directory cleaned: {}", dir);
                } else {
                    error!("Failed to clean directory: {}", dir);
                }
            }
        }
    }
}

impl Config {
    /// Retrieve a tier by its ID.
    pub fn get_tier(&self, id: &str) -> Option<&Tier> {
        self.tiers.iter().find(|tier| tier.id == id)
    }
}

/// Load the configuration from a file or use defaults.
pub fn load_config(config_file: Option<&Path>) -> Config {
    let mut config = Config::default();
    if let Some(config_file) = config_file {
        let config_file = read_config_file(config_file).unwrap_or_default();
        parse_network_config(&mut config, &config_file);
        parse_auth_config(&mut config, &config_file);
        parse_storage_config(&mut config, &config_file);
        parse_auth_config(&mut config, &config_file);
        parse_tiers_config(&mut config, &config_file);
        parse_federation_config(&mut config, &config_file);
    }

    if logger::init_logger(&config.storage.log_dir, LevelFilter::Debug).is_err() {
        exit(1);
    }

    if config_file.is_some() {
        info!(
            "Loading configuration from: {}",
            config_file.unwrap().display()
        );
    } else {
        warn!("No configuration file provided, using default settings");
    }

    config.storage.init();
    CONFIG.get_or_init(|| config.clone());
    config
}

/// Read the configuration file and parse it as a TOML table.
fn read_config_file(config_file: &Path) -> Result<Table, String> {
    let file_content = read_file_to_string(config_file).map_err(|e| e.to_string())?;
    toml::from_str(&file_content).map_err(|e| e.to_string())
}

/// Parse the network configuration.
fn parse_network_config(config: &mut Config, table: &Table) {
    if let Some(network) = table.get("network").and_then(|v| v.as_table()) {
        if let Some(port) = network.get("port").and_then(|v| v.as_integer()) {
            config.network.port = port as u16;
        }
        if let Some(host) = network.get("host").and_then(|v| v.as_str()) {
            config.network.host = host.to_string();
        }
    }
}

/// Parse the storage configuration.
fn parse_storage_config(config: &mut Config, table: &Table) {
    if let Some(storage) = table.get("storage").and_then(|v| v.as_table()) {
        if let Some(data_dir) = storage.get("data_dir").and_then(|v| v.as_str()) {
            config.storage.data_dir = data_dir.to_string();
        }
        if let Some(temp_dir) = storage.get("temp_dir").and_then(|v| v.as_str()) {
            config.storage.temp_dir = temp_dir.to_string();
        }
        if let Some(log_dir) = storage.get("log_dir").and_then(|v| v.as_str()) {
            config.storage.log_dir = log_dir.to_string();
        }
    }
}

/// Parse the authentication configuration.
fn parse_auth_config(config: &mut Config, table: &Table) {
    if let Some(auth) = table.get("auth").and_then(|v| v.as_table()) {
        if let Some(session_timeout) = auth.get("session_timeout").and_then(|v| v.as_integer()) {
            config.auth.session_timeout = session_timeout as u64;
        }
        if let Some(auth_session_timeout) = auth
            .get("auth_session_timeout")
            .and_then(|v| v.as_integer())
        {
            config.auth.auth_session_timeout = auth_session_timeout as u64;
        }
        if let Some(registration_enabled) =
            auth.get("registration_enabled").and_then(|v| v.as_bool())
        {
            config.auth.registration_enabled = registration_enabled;
        }
    }
}

/// Parse the tiers configuration.
fn parse_tiers_config(config: &mut Config, table: &Table) {
    if let Some(tiers) = table.get("tiers").and_then(|v| v.as_array()) {
        config.tiers.clear();
        for tier in tiers {
            if let Some(tier_table) = tier.as_table() {
                if let (Some(id), Some(name), Some(description), Some(quota)) = (
                    tier_table.get("id").and_then(|v| v.as_str()),
                    tier_table.get("name").and_then(|v| v.as_str()),
                    tier_table.get("description").and_then(|v| v.as_str()),
                    tier_table.get("quota").and_then(|v| v.as_integer()),
                ) {
                    config.tiers.push(Tier {
                        id: id.to_string(),
                        name: name.to_string(),
                        description: description.to_string(),
                        quota: quota as u64,
                    });
                }
            }
        }
    }
}

/// Parse the federation configuration.
fn parse_federation_config(config: &mut Config, table: &Table) {
    if let Some(federation) = table.get("federation").and_then(|v| v.as_table()) {
        config.federation.enabled = federation
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if let Some(blacklist) = federation.get("blacklist").and_then(|v| v.as_table()) {
            config.federation.blacklist.enabled = blacklist
                .get("enable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if let Some(hosts) = blacklist.get("hosts").and_then(|v| v.as_array()) {
                config.federation.blacklist.hosts = hosts
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }

        if let Some(whitelist) = federation.get("whitelist").and_then(|v| v.as_table()) {
            config.federation.whitelist.enabled = whitelist
                .get("enable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if let Some(hosts) = whitelist.get("hosts").and_then(|v| v.as_array()) {
                config.federation.whitelist.hosts = hosts
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }
    }
}

/// Get the current configuration, returning defaults if not set.
pub fn get_config() -> Config {
    let default_config = Config::default();
    CONFIG.get().unwrap_or(&default_config).to_owned()
}
