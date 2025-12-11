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
    pub id: ShareID,
    pub blobs: Vec<BlobID>,
    pub write: bool,
    pub secret: ShareSecret,
}

#[derive(Debug, Deserialize, Clone, Copy, Serialize, PartialEq)]
pub struct ShareSecret(u128);
#[derive(Debug, Deserialize, Clone, Copy, Serialize, PartialEq)]
pub struct ShareID(u128);
#[derive(Debug, Deserialize, Clone, Copy, Serialize, PartialEq)]
pub struct BlobID(u128);
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub struct BlobHash(String);
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct BlobContent(Vec<u8>);
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Base64(String);

impl From<BlobID> for String {
    fn from(val: BlobID) -> Self {
        u128_to_32_char_hex_string(val.0)
    }
}

impl From<String> for BlobID {
    fn from(val: String) -> Self {
        BlobID(char_hex_string_to_u128(val))
    }
}

impl From<String> for ShareID {
    fn from(val: String) -> Self {
        ShareID(char_hex_string_to_u128(val))
    }
}

impl From<String> for ShareSecret {
    fn from(val: String) -> Self {
        ShareSecret(char_hex_string_to_u128(val))
    }
}

impl From<String> for Base64 {
    fn from(val: String) -> Self {
        Base64(val)
    }
}

impl BlobHash {
    pub fn hash(data: Vec<u8>) -> Self {
        BlobHash(sha256::digest(data))
    }
}

impl From<String> for BlobHash {
    fn from(val: String) -> Self {
        BlobHash(val)
    }
}

impl From<BlobHash> for String {
    fn from(val: BlobHash) -> Self {
        val.0
    }
}

pub fn base64_encode(data: Vec<u8>) -> Base64 {
    Base64(base64::engine::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        data,
    ))
}
pub fn base64_decode(data: Base64) -> Result<Vec<u8>, Error> {
    match base64::engine::Engine::decode(&base64::engine::general_purpose::STANDARD, data.0) {
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

    fn resolve_blob_path(&self, id: BlobID) -> String {
        let string: String = id.into();
        self.resolve_data_path("blobs/") + string.as_str()
    }

    pub fn create_blob(&self, content: Base64) -> Result<(BlobID, BlobHash), Error> {
        let data = base64_decode(content)?;
        let available_quota = self.get_available_quota();
        if available_quota < data.len() as u64 {
            return Err(Error::new(ERROR_QUOTA_EXCEEDED));
        }
        self.create_blob_dir();
        let mut id = BlobID(random_u128());
        while file_exists(self.resolve_blob_path(id).as_str()) {
            id = BlobID(random_u128());
        }

        write_file(self.resolve_blob_path(id).as_str(), data.to_owned());
        Ok((id, BlobHash::hash(data)))
    }

    pub fn read_blob(&self, id: BlobID) -> Result<(Base64, BlobHash), Error> {
        self.create_blob_dir();
        let path = self.resolve_blob_path(id);
        if !file_exists(path.as_str()) {
            return Err(Error::new(ERROR_BLOB_NOT_FOUND));
        }
        match read_file(path.as_str()) {
            Ok(content) => Ok((base64_encode(content.to_owned()), BlobHash::hash(content))),
            Err(_) => Err(Error::new(ERROR_BLOB_NOT_FOUND)),
        }
    }

    pub fn update_blob(
        &self,
        id: BlobID,
        content: Base64,
        old_hash: BlobHash,
    ) -> Result<BlobHash, Error> {
        self.create_blob_dir();
        let path = self.resolve_blob_path(id);
        if !file_exists(path.as_str()) {
            return Err(Error::new(ERROR_BLOB_NOT_FOUND));
        }
        match read_file(path.as_str()) {
            Ok(old_content) => {
                let data = base64_decode(content)?;
                let hash = BlobHash::hash(old_content);
                if old_hash != hash {
                    return Err(Error::new(ERROR_BLOB_HASH_NOT_MATCH));
                }
                let available_quota = self.get_available_quota();
                if available_quota < data.len() as u64 {
                    return Err(Error::new(ERROR_QUOTA_EXCEEDED));
                }
                write_file(path.as_str(), data.to_owned());
                Ok(BlobHash::hash(data))
            }
            Err(_) => Err(Error::new(ERROR_BLOB_NOT_FOUND)),
        }
    }

    pub fn delete_blob(&self, id: BlobID) -> bool {
        self.create_blob_dir();
        let path = self.resolve_blob_path(id);
        if !file_exists(path.as_str()) {
            return false;
        }
        remove_file(path.as_str());
        let _ = self.delete_shared_blob(id);
        true
    }

    fn get_share_data(&self) -> Vec<Share> {
        serde_json::from_str(
            read_file_to_string(self.resolve_data_path("shares.json").as_str())
                .unwrap_or("[]".to_string())
                .as_str(),
        )
        .unwrap_or(vec![])
    }

    fn set_share_data(&self, shares: Vec<Share>) -> bool {
        write_file_from_string(
            self.resolve_data_path("shares.json").as_str(),
            serde_json::to_string(&shares)
                .unwrap_or("[]".to_string())
                .as_str(),
        )
    }

    pub fn check_share_permissions(
        &self,
        id: ShareID,
        secret: ShareSecret,
        blob_id: BlobID,
        write: bool,
    ) -> Result<(), Error> {
        let share = self.get_share_by_id(id)?;
        if share.secret != secret {
            Err(Error::new(ERROR_WRONG_SECRET))
        } else if !share.blobs.iter().any(|b| *b == blob_id) {
            Err(Error::new(ERROR_BLOB_NOT_IN_SHARE))
        } else if write && !share.write {
            Err(Error::new(ERROR_NO_WRITE_ACCESS))
        } else {
            Ok(())
        }
    }

    pub fn get_share_by_id(&self, id: ShareID) -> Result<Share, Error> {
        let shares = self.get_share_data();
        let share = shares
            .iter()
            .find(|i| i.id == id)
            .ok_or_else(|| Error::new(ERROR_SHARE_NOT_FOUND))?;
        Ok(share.to_owned())
    }

    pub fn delete_shared_blob(&self, blob: BlobID) -> Result<(), Error> {
        let mut shares = self.get_share_data();
        for share in &mut shares {
            for i in 0..share.blobs.len() {
                if share.blobs[i] == blob {
                    share.blobs.remove(i);
                }
            }
        }
        self.set_share_data(shares);
        Ok(())
    }

    pub fn add_blob_to_share(&self, share_id: ShareID, blob: BlobID) -> Result<(), Error> {
        let mut shares = self.get_share_data();
        for share in &mut shares {
            if share.id == share_id {
                share.blobs.push(blob);
                break;
            }
        }
        self.set_share_data(shares);
        Ok(())
    }

    pub fn validate_share_access(
        &self,
        share_id: ShareID,
        share_secret: ShareSecret,
    ) -> Result<Share, Error> {
        let share = self.get_share_by_id(share_id)?;
        if share.secret != share_secret {
            return Err(Error::new(ERROR_WRONG_SECRET));
        }
        Ok(share)
    }

    pub fn validate_blob_access(
        &self,
        share_id: ShareID,
        share_secret: ShareSecret,
        blob: BlobID,
        write_access: bool,
    ) -> Result<(), Error> {
        self.validate_share_access(share_id, share_secret)?;
        self.check_share_permissions(share_id, share_secret, blob, write_access)
    }
}
