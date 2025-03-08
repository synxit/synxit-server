mod blob;
mod sessions;

use crate::storage::file::{read_file, write_file, read_dir};
use serde::{Deserialize, Serialize};

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
    pub mfa_count: u8,
    pub password_correct: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(skip)]
    pub username: String,
    pub email: String,
    pub sessions: Vec<Session>,
    pub auth: Auth
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MFAMethod {
    pub id: u16,
    pub name: String,
    pub enabled: bool,
    pub data: String,
    pub r#type: MFAMethodType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MFAMethodType {
    #[serde(rename = "totp")]
    TOTP,
    #[serde(rename = "u2f")]
    U2F,
}

impl User {
    pub fn all() -> Vec<User> {
        let mut users = vec![];
        for user in read_dir(
            &(CONFIG.get().unwrap().storage.data_dir.to_string() + "/users/"),
            false
        ) {
            users.push(User::load(("@".to_owned() + user.as_str() + ":").as_str()));
        }
        users
    }

    pub fn new(username: String, email: String) -> User {
        User {
            username,
            email,
            sessions: vec![],
            auth: Auth {
                hash: String::new(),
                salt: String::new(),
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

    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize user")
    }

    pub fn from_json(json: &str) -> User {
        serde_json::from_str(json).expect("Failed to deserialize user")
    }

    pub fn save(&self) {
        write_file(
            self.resolve_data_path("data.json").as_str(),
            &self.to_string(),
        )
    }

    pub fn load(username: &str) -> User {
        let data = read_file(Self::resolve_user_data_path(username, "data.json").as_str());
        let mut user = User::from_json(data.as_str());
        user.username = username.to_string();
        user
    }
    pub fn resolve_data_path(&self, path: &str) -> String {
        Self::resolve_user_data_path(self.username.as_str(), path)
    }

    pub fn resolve_user_data_path(username: &str, path: &str) -> String {
        CONFIG.get().unwrap().storage.data_dir.to_string()
            + "/users/"
            + username.split("@").collect::<Vec<&str>>()[1].split(":").collect::<Vec<&str>>()[0]
            + "/"
            + path
    }
}
