mod blob;
mod sessions;

use crate::logger::error::Error;
use crate::storage::file::{file_exists, read_dir, read_file, write_file};
use log::{error, warn};
use serde::{Deserialize, Serialize};
use totp_rs::{Secret, TOTP};

use super::config::CONFIG;

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
    pub recovery_codes: Vec<String>,
    pub min_methods: u8,
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
            &(CONFIG.get().unwrap().storage.data_dir.to_string() + "/users/"),
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

    pub fn new(username: String, hash: String, salt: String) -> User {
        User {
            username,
            sessions: vec![],
            auth: Auth {
                hash,
                salt,
                auth_sessions: vec![],
                mfa: MFA {
                    enabled: false,
                    methods: vec![],
                    recovery_codes: vec![],
                    min_methods: 0,
                },
                encrypted: EncryptedData {
                    master_key: String::new(),
                    keyring: String::new(),
                    blob_map: String::new(),
                },
            },
        }
    }

    pub fn user_exists(username: &str) -> bool {
        !file_exists(Self::resolve_user_data_path(username, "data.json"))
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize user")
    }

    pub fn from_json(json: &str) -> User {
        serde_json::from_str(json).expect("Failed to deserialize user")
    }

    pub fn save(&self) -> bool {
        if write_file(
            self.resolve_data_path("data.json").as_str(),
            &self.to_string(),
        ) {
            true
        } else {
            error!("Error saving user data");
            false
        }
    }

    pub fn load(username: &str) -> Result<User, Error> {
        let lower_username = username.to_lowercase();
        match read_file(Self::resolve_user_data_path(username, "data.json").as_str()) {
            Ok(data) => {
                let mut user = User::from_json(data.as_str());
                user.username = lower_username;
                Ok(user)
            }
            Err(_) => {
                warn!("Error loading user data");
                Err(Error::new("Could not load user data"))
            }
        }
    }

    pub fn resolve_data_path(&self, path: &str) -> String {
        Self::resolve_user_data_path(self.username.as_str(), path)
    }

    pub fn resolve_user_data_path(username: &str, path: &str) -> String {
        CONFIG.get().unwrap().storage.data_dir.to_string()
            + "/users/"
            + username.split("@").collect::<Vec<&str>>()[1]
                .split(":")
                .collect::<Vec<&str>>()[0]
            + "/"
            + path
    }

    pub fn create_mfa(&mut self, r#type: MFAMethodType) -> MFAMethod {
        match r#type {
            MFAMethodType::TOTP => {
                // rand u16
                let mut free_mfa_id = rand::random::<u8>();
                while self.auth.mfa.methods.iter().any(|m| m.id == free_mfa_id) {
                    free_mfa_id = rand::random::<u8>();
                }
                let method = MFAMethod {
                    id: free_mfa_id,
                    name: "TOTP".to_string(),
                    enabled: true,
                    data: TOTP::default().get_secret_base32(),
                    r#type: MFAMethodType::TOTP,
                };
                self.auth.mfa.methods.push(method.clone());
                method
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
                method
            }
        }
    }

    pub fn check_mfa(&self, id: u8, code: &str) -> bool {
        let method = self.auth.mfa.methods.iter().find(|m| m.id == id);
        if let Some(method) = method {
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
    }
}
