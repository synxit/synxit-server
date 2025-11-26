use crate::{
    logger::error::{
        Error, ERROR_BLOB_HASH_NOT_MATCH, ERROR_BLOB_NOT_FOUND, ERROR_BLOB_NOT_IN_SHARE,
        ERROR_NO_WRITE_ACCESS, ERROR_QUOTA_EXCEEDED, ERROR_SHARE_NOT_FOUND, ERROR_WRONG_SECRET,
    },
    storage::file::{
        create_dir, dir_exists, file_exists, read_file, read_file_to_string, remove_file,
        write_file, write_file_from_string,
    },
    utils::{char_hex_string_to_u128, random_u128, u128_to_32_char_hex_string},
    User,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Share {
    pub id: u128,
    pub blobs: Vec<String>,
    pub write: bool,
    pub secret: u128,
}

pub fn base64_encode(data: Vec<u8>) -> String {
    base64::engine::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}
pub fn base64_decode(data: &str) -> Result<Vec<u8>, Error> {
    match base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, data) {
        Ok(decoded) => Ok(decoded),
        Err(e) => Err(Error::new(format!("Base64 decode error: {}", e).as_str())),
    }
}

impl User {
    fn create_blob_dir(&self) {
        if !dir_exists(self.resolve_data_path("blobs/").as_str()) {
            create_dir(self.resolve_data_path("blobs/").as_str());
        }
    }

    fn is_valid_blob_id(id: &str) -> bool {
        id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn resolve_blob_path(&self, id: &str) -> String {
        self.resolve_data_path("blobs/") + id
    }

    pub fn create_blob(&self, content: &str) -> Result<(String, String), Error> {
        let data = base64_decode(content)?;
        let available_quota = self.get_available_quota();
        if available_quota < content.len() as u64 {
            return Err(Error::new(ERROR_QUOTA_EXCEEDED));
        }
        self.create_blob_dir();
        let mut id = u128_to_32_char_hex_string(random_u128());
        while file_exists(self.resolve_blob_path(id.as_str()).as_str())
            && !Self::is_valid_blob_id(id.as_str())
        {
            id = u128_to_32_char_hex_string(random_u128());
        }
        write_file(
            self.resolve_blob_path(id.as_str()).as_str(),
            data.to_owned(),
        );
        Ok((id, sha256::digest(data).to_string()))
    }

    pub fn read_blob(&self, id: &str) -> Result<(String, String), Error> {
        self.create_blob_dir();
        if !Self::is_valid_blob_id(id) {
            return Err(Error::new(ERROR_BLOB_NOT_FOUND));
        }
        let path = self.resolve_blob_path(id);
        if !file_exists(path.as_str()) {
            return Err(Error::new(ERROR_BLOB_NOT_FOUND));
        }
        match read_file(path.as_str()) {
            Ok(content) => Ok((
                base64_encode(content.to_owned()),
                sha256::digest(content).to_string(),
            )),
            Err(_) => Err(Error::new(ERROR_BLOB_NOT_FOUND)),
        }
    }

    pub fn update_blob(&self, id: &str, content: &str, old_hash: &str) -> Result<String, Error> {
        self.create_blob_dir();
        if !Self::is_valid_blob_id(id) {
            return Err(Error::new(ERROR_BLOB_NOT_FOUND));
        }
        let path = self.resolve_blob_path(id);
        if !file_exists(path.as_str()) {
            return Err(Error::new(ERROR_BLOB_NOT_FOUND));
        }
        match read_file(path.as_str()) {
            Ok(old_content) => {
                let data = base64_decode(content)?;
                let hash = sha256::digest(old_content);
                if old_hash != hash {
                    return Err(Error::new(ERROR_BLOB_HASH_NOT_MATCH));
                }
                let available_quota = self.get_available_quota();
                if available_quota < content.len() as u64 {
                    return Err(Error::new(ERROR_QUOTA_EXCEEDED));
                }
                write_file(path.as_str(), data.to_owned());
                Ok(sha256::digest(data).to_string())
            }
            Err(_) => Err(Error::new(ERROR_BLOB_NOT_FOUND)),
        }
    }

    pub fn delete_blob(&self, id: &str) -> bool {
        self.create_blob_dir();
        if !Self::is_valid_blob_id(id) {
            return false;
        }
        let path = self.resolve_blob_path(id);
        if !file_exists(path.as_str()) {
            return false;
        }
        remove_file(path.as_str());
        let _ = Self::delete_shared_blob(self.username.to_string(), id.to_string());
        true
    }

    fn get_share_data(username: String) -> Result<Vec<Share>, Error> {
        match Self::resolve_user_data_path(username.as_str(), "shares.json") {
            Ok(path) => Ok(serde_json::from_str(
                read_file_to_string(path)
                    .unwrap_or("[]".to_string())
                    .as_str(),
            )
            .unwrap_or(vec![])),
            Err(e) => Err(e),
        }
    }

    fn set_share_data(username: String, shares: Vec<Share>) -> bool {
        match Self::resolve_user_data_path(username.as_str(), "shares.json") {
            Ok(path) => {
                write_file_from_string(
                    path,
                    serde_json::to_string(&shares)
                        .unwrap_or("[]".to_string())
                        .as_str(),
                );
                true
            }
            Err(_) => false,
        }
    }

    pub fn check_share_permissions(
        username: String,
        id: String,
        secret: String,
        blob_id: String,
        write: bool,
    ) -> Result<(), Error> {
        let share = Self::get_share_by_id(username, id)?;

        let secret_num = char_hex_string_to_u128(secret);
        if secret_num != share.secret {
            Err(Error::new(ERROR_WRONG_SECRET))
        } else if !share.blobs.iter().any(|b| *b == blob_id) {
            Err(Error::new(ERROR_BLOB_NOT_IN_SHARE))
        } else if write && !share.write {
            Err(Error::new(ERROR_NO_WRITE_ACCESS))
        } else {
            Ok(())
        }
    }

    pub fn get_share_by_id(username: String, id: String) -> Result<Share, Error> {
        let id_num = char_hex_string_to_u128(id);
        let shares = Self::get_share_data(username)?;
        let share = shares
            .iter()
            .find(|i| i.id == id_num)
            .ok_or_else(|| Error::new(ERROR_SHARE_NOT_FOUND))?;
        Ok(share.to_owned())
    }

    pub fn delete_shared_blob(username: String, blob: String) -> Result<(), Error> {
        let mut shares = Self::get_share_data(username.to_owned())?;
        for share in &mut shares {
            for i in 0..share.blobs.len() {
                if share.blobs[i] == blob {
                    share.blobs.remove(i);
                }
            }
        }
        Self::set_share_data(username, shares);
        Ok(())
    }

    pub fn add_blob_to_share(
        username: String,
        share_id: String,
        blob: String,
    ) -> Result<(), Error> {
        let mut shares = Self::get_share_data(username.to_owned())?;
        for share in &mut shares {
            if share.id == char_hex_string_to_u128(share_id.to_string()) {
                share.blobs.push(blob);
                break;
            }
        }
        Self::set_share_data(username, shares);
        Ok(())
    }
}
