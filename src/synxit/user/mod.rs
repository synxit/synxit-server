pub mod blob;
mod sessions;

use crate::logger::error::Error;
use crate::storage::file::{
    create_dir, dir_exists, file_exists, get_folder_size, read_dir, read_file, write_file,
};
use crate::synxit::config::Config;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use totp_rs::{Secret, TOTP};

use super::config::{get_config, CONFIG};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: u128,
    pub created_at: u64,
    pub last_used: u64,
    pub root: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: u128,
    pub expires_at: u64,
    pub challenge: u128,
    pub completed_mfa: Vec<u8>,
    pub password_correct: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(skip)]
    pub username: String,
    pub sessions: Vec<Session>,
    pub auth: Auth,
    pub tier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedData {
    pub master_key: String,
    pub keyring: String,
    pub blob_map: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub hash: String,
    pub salt: String,
    pub auth_sessions: Vec<AuthSession>,
    pub mfa: MFA,
    pub encrypted: EncryptedData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MFA {
    pub enabled: bool,
    pub methods: Vec<MFAMethod>,
    pub recovery_codes: [String; 8],
    pub min_methods: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MFAMethodPublic {
    pub id: u8,
    pub name: String,
    pub r#type: MFAMethodType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MFAMethod {
    pub id: u8,
    pub name: String,
    pub enabled: bool,
    pub data: String,
    pub r#type: MFAMethodType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MFAMethodType {
    #[serde(rename = "totp")]
    TOTP,
    #[serde(rename = "u2f")]
    U2F,
}

impl User {
    pub fn all() -> Vec<User> {
        let mut users = vec![];
        match read_dir(
            &(get_config().storage.data_dir.to_string() + "/users/"),
            false,
        ) {
            Ok(dir) => {
                for user in dir {
                    match User::load(("@".to_owned() + user.as_str() + ":").as_str()) {
                        Ok(user) => users.push(user),
                        Err(err) => error!("Error loading user: {}", err),
                    }
                }
            }
            Err(err) => error!("Error reading users directory: {}", err),
        }
        users
    }

    pub fn new(username: &str, hash: &str, salt: &str) -> User {
        User {
            username: username.to_string(),
            sessions: vec![],
            auth: Auth {
                hash: hash.to_string(),
                salt: salt.to_string(),
                auth_sessions: vec![],
                mfa: MFA {
                    enabled: false,
                    methods: vec![],
                    recovery_codes: [
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                    ],
                    min_methods: 0,
                },
                encrypted: EncryptedData {
                    master_key: String::new(),
                    keyring: String::new(),
                    blob_map: String::new(),
                },
            },
            tier: String::new(),
        }
    }

    pub fn user_exists(username: &str) -> bool {
        match Self::resolve_user_data_path(username, "data.json") {
            Ok(path) => file_exists(path),
            Err(_) => false,
        }
    }

    pub fn to_string(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::new(format!("Error serializing user data: {}", e).as_str()))
    }

    pub fn from_json(json: &str) -> Result<User, Error> {
        serde_json::from_str(json)
            .map_err(|e| Error::new(format!("Error parsing user data: {}", e).as_str()))
    }

    /// Save the user data to the disk
    pub fn save(&self) -> bool {
        if !dir_exists(self.resolve_data_path("")) {
            if !create_dir(self.resolve_data_path("")) {
                error!("Error creating user directory");
                return false;
            }
            info!("User directory created {}", self.resolve_data_path(""));
        }
        match &self.to_string() {
            Ok(string) => {
                if write_file(self.resolve_data_path("data.json").as_str(), string) {
                    true
                } else {
                    error!("Error saving user data");
                    false
                }
            }
            Err(err) => {
                error!("Error serializing user data: {}", err);
                false
            }
        }
    }

    pub fn load(username: &str) -> Result<User, Error> {
        let lower_username = username.to_lowercase();
        match Self::resolve_user_data_path(username, "data.json") {
            Ok(path) => match read_file(path) {
                Ok(data) => {
                    match User::from_json(data.as_str()) {
                        Ok(mut user) => {
                            user.username = lower_username;
                            Ok(user)
                        }
                        Err(err) => {
                            warn!("Error parsing user data: {}", err);
                            return Err(Error::new("Could not parse user data"));
                        }
                    }
                }
                Err(_) => {
                    warn!("Error loading user data");
                    Err(Error::new("Could not load user data"))
                }
            },
            Err(_) => {
                warn!("Error loading user data");
                Err(Error::new("Could not load user data"))
            }
        }
    }

    pub fn resolve_data_path(&self, path: &str) -> String {
        // I know unwrap is not the best practice here, but we are sure that the path is valid
        match Self::resolve_user_data_path(self.username.as_str(), path) {
            Ok(path) => path,
            Err(err) => {
                warn!("Error resolving data path: {}", err);
                "".to_string()
            }
        }
    }

    pub fn resolve_user(user: &str) -> Result<(String, String), Error> {
        // check if username start with "@"
        let username = user.to_lowercase();
        if username.starts_with('@') {
            if username.contains(":") {
                let vec: Vec<&str> = username.trim_start_matches('@').split(":").collect();
                Ok((vec[0].to_string(), vec[1].to_string()))
            } else {
                let vec: Vec<&str> = username.split("@").collect();
                Ok(("root".to_string(), vec[1].to_string()))
            }
        } else {
            Err(Error::new("Invalid username, not starting with '@'"))
        }
    }

    pub fn resolve_user_data_path(username: &str, path: &str) -> Result<String, Error> {
        match Self::resolve_user(username) {
            Ok(user) => Ok(CONFIG.get().unwrap().storage.data_dir.to_string()
                + "/users/"
                + user.0.as_str()
                + "/"
                + path),
            Err(err) => Err(err),
        }
    }

    pub fn create_mfa(&mut self, r#type: MFAMethodType, name: String) -> Option<MFAMethod> {
        match r#type {
            MFAMethodType::TOTP => {
                let mut free_mfa_id = rand::random::<u8>();
                if self.auth.mfa.methods.len() >= 255 {
                    error!("Too many MFA methods");
                    return None;
                }
                while self.auth.mfa.methods.iter().any(|m| m.id == free_mfa_id) && free_mfa_id < 255
                {
                    free_mfa_id = rand::random::<u8>();
                }
                let method = MFAMethod {
                    name,
                    id: free_mfa_id,
                    enabled: true,
                    data: TOTP::default().get_secret_base32(),
                    r#type: MFAMethodType::TOTP,
                };
                self.auth.mfa.methods.push(method.clone());
                Some(method)
            }
            MFAMethodType::U2F => {
                // rand u16
                let mut free_mfa_id = rand::random::<u8>();
                while self.auth.mfa.methods.iter().any(|m| m.id == free_mfa_id) {
                    free_mfa_id = rand::random::<u8>();
                }
                let method = MFAMethod {
                    id: free_mfa_id,
                    name: "U2F".to_string(),
                    enabled: true,
                    data: "".to_string(),
                    r#type: MFAMethodType::U2F,
                };
                self.auth.mfa.methods.push(method.clone());
                Some(method)
            }
        }
    }

    pub fn check_mfa_recovery_code(&mut self, code: &str) -> bool {
        if let Some(pos) = self.auth.mfa.recovery_codes.iter().position(|c| c == code) {
            self.auth.mfa.recovery_codes[pos] = String::new();
            true
        } else {
            false
        }
    }

    pub fn check_mfa(&self, id: u8, code: &str) -> bool {
        let method = self.auth.mfa.methods.iter().find(|m| m.id == id);
        if let Some(method) = method {
            if method.enabled {
                match method.r#type {
                    MFAMethodType::TOTP => match Secret::Encoded(method.clone().data).to_bytes() {
                        Ok(bytes) => match TOTP::new(totp_rs::Algorithm::SHA1, 6, 1, 30, bytes) {
                            Ok(totp) => totp.check_current(code).unwrap_or(false),
                            Err(_) => false,
                        },
                        Err(_) => false,
                    },
                    MFAMethodType::U2F => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_tier_quota(&self) -> u64 {
        if CONFIG.get().is_some() {
            let default_config = Config::default();
            match CONFIG
                .get()
                .unwrap_or(&default_config)
                .get_tier(self.tier.as_str())
            {
                Some(tier) => tier.quota,
                None => u64::MAX,
            }
        } else {
            0
        }
    }

    pub fn get_used_quota(&self) -> u64 {
        match get_folder_size(self.resolve_data_path("")) {
            Ok(size) => size,
            Err(_) => 0,
        }
    }

    pub fn get_available_quota(&self) -> u64 {
        self.get_tier_quota()
            .checked_sub(self.get_used_quota())
            .unwrap_or(0)
    }

    fn generate_recovery_code() -> String {
        let code: String = (0..10)
            .map(|_| {
                let idx = rand::random::<usize>() % 36;
                if idx < 10 {
                    (b'0' + idx as u8) as char
                } else {
                    (b'A' + (idx - 10) as u8) as char
                }
            })
            .collect();
        code
    }

    pub fn generate_recovery_codes(&mut self) -> [String; 8] {
        let mut codes: [String; 8] = [
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ];
        for i in 0..8 {
            let code = Self::generate_recovery_code();
            codes[i] = code.to_string();
        }
        self.auth.mfa.recovery_codes = codes.to_owned();
        codes
    }
}
