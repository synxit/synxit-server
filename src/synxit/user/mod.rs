pub mod blob;
mod sessions;

use std::fmt::Display;

use crate::logger::error::Error;
use crate::storage::file::{
    create_dir, dir_exists, file_exists, get_folder_size, read_dir, read_file_to_string,
    write_file_from_string,
};
use crate::synxit::config::Config;
use crate::utils::{char_hex_string_to_u128, u128_to_32_char_hex_string};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use totp_rs::TOTP;

use super::config::{get_config, CONFIG};
use super::security::verify_totp_code;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionID,
    pub created_at: u64,
    pub last_used: u64,
    pub root: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: AuthSessionID,
    pub expires_at: u64,
    pub challenge: u128,
    pub completed_mfa: Vec<u8>,
    pub password_correct: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(skip)]
    pub userhandle: UserHandle,
    pub sessions: Vec<Session>,
    pub auth: Auth,
    pub foreign_keyring: String,
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Username(String);
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server(String);
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct UserHandle(String);

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct AuthSessionID(u128);
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct SessionID(u128);

impl From<SessionID> for String {
    fn from(val: SessionID) -> Self {
        u128_to_32_char_hex_string(val.0)
    }
}

impl From<String> for SessionID {
    fn from(val: String) -> Self {
        SessionID(char_hex_string_to_u128(val))
    }
}

impl From<AuthSessionID> for String {
    fn from(val: AuthSessionID) -> Self {
        u128_to_32_char_hex_string(val.0)
    }
}

impl From<String> for AuthSessionID {
    fn from(val: String) -> Self {
        AuthSessionID(char_hex_string_to_u128(val))
    }
}

impl Server {
    pub fn new(s: String) -> Self {
        Server(s)
    }
}

impl Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Display for UserHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.split();
        if s.0 .0 == "root" {
            write!(f, "@{}:", s.1 .0)
        } else {
            write!(f, "@{}:{}", s.0 .0, s.1 .0)
        }
    }
}

impl UserHandle {
    pub fn get_local_username(&self) -> String {
        self.split().0 .0
    }

    pub fn get_server(&self) -> Server {
        self.split().1
    }

    fn split(&self) -> (Username, Server) {
        // remove @ in front only
        let s = self.0.clone().to_lowercase().replace("@", "");
        if s.contains(":") {
            let parts: Vec<&str> = s.split(':').collect();
            (Username(parts[0].to_string()), Server(parts[1].to_string()))
        } else {
            (
                Username("root".to_string()),
                Server(self.0.clone())
            )
        }
    }

    fn verify(&self) -> bool {
        let s = self.split();
        s.0.0.chars().all(|c| c.is_ascii_alphanumeric() || c == '.') &&
        s.1.0.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') &&
        s.0.0.len() >= 3 && s.0.0.len() <= 32 && // username length
        s.1.0.len() >= 3 && s.1.0.len() <= 253 // domain name max length
    }

    pub fn from_string(s: String) -> Result<Self, Error> {
        let tmp = Self(s);
        if tmp.verify() {
            Ok(tmp)
        } else {
            Err(Error::new("Invalid userhandle string"))
        }
    }
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
                    match UserHandle::from_string("@".to_string() + user.as_str() + ":localhost") {
                        Ok(userhandle) => match User::load(userhandle) {
                            Ok(loaded_user) => users.push(loaded_user),
                            Err(err) => warn!("Error loading user {}: {}", user, err),
                        },
                        Err(err) => {
                            warn!(
                                "Error parsing userhandle {}: {}",
                                "@".to_string() + user.as_str() + ":localhost",
                                err
                            );
                        }
                    }
                }
            }
            Err(err) => error!("Error reading users directory: {}", err),
        }
        users
    }

    pub fn new(userhandle: UserHandle, hash: &str, salt: &str) -> User {
        User {
            userhandle,
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
            foreign_keyring: String::new(),
            tier: String::new(),
        }
    }

    pub fn user_exists(userhandle: UserHandle) -> bool {
        file_exists(Self::resolve_user_data_path(userhandle, "data.json"))
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
                if write_file_from_string(self.resolve_data_path("data.json").as_str(), string) {
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

    pub fn load(userhandle: UserHandle) -> Result<User, Error> {
        match read_file_to_string(Self::resolve_user_data_path(
            userhandle.to_owned(),
            "data.json",
        )) {
            Ok(data) => match User::from_json(data.as_str()) {
                Ok(mut user) => {
                    user.userhandle = userhandle;
                    Ok(user)
                }
                Err(err) => {
                    warn!("Error parsing user data: {}", err);
                    Err(Error::new("Could not parse user data"))
                }
            },
            Err(_) => {
                warn!("Error loading user data");
                Err(Error::new("Could not load user data"))
            }
        }
    }

    pub fn resolve_data_path(&self, path: &str) -> String {
        Self::resolve_user_data_path(self.userhandle.to_owned(), path)
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

    fn resolve_user_data_path(username: UserHandle, path: &str) -> String {
        CONFIG.get().unwrap().storage.data_dir.to_string()
            + "/users/"
            + username.get_local_username().as_str()
            + "/"
            + path
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
                    MFAMethodType::TOTP => verify_totp_code(method.data.to_string(), code),
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
        get_folder_size(self.resolve_data_path("")).unwrap_or_default()
    }

    pub fn get_available_quota(&self) -> u64 {
        self.get_tier_quota().saturating_sub(self.get_used_quota())
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
        let codes: [String; 8] = std::array::from_fn(|_| Self::generate_recovery_code());
        self.auth.mfa.recovery_codes = codes.to_owned();
        codes
    }
}
