use crate::{
    storage::file::{create_dir, dir_exists, file_exists, read_file, remove_file, write_file},
    utils::{random_u128, u128_to_32_char_hex_string},
    User,
};
use serde::{Deserialize, Serialize};
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

    pub fn new_blob(&self, content: &str) -> BlobResponse {
        self.create_blob_dir();
        let mut id = String::new();
        while file_exists(self.resolve_blob_path(id.as_str()).as_str())
            && !Self::is_valid_blob_id(id.as_str())
        {
            id = u128_to_32_char_hex_string(random_u128());
        }
        write_file(self.resolve_blob_path(id.as_str()).as_str(), content);
        BlobResponse {
            success: true,
            id: id,
            content: "".to_string(),
            hash: sha256::digest(content).to_string(),
        }
    }

    pub fn read_blob(&self, id: &str) -> BlobResponse {
        self.create_blob_dir();
        if !Self::is_valid_blob_id(id) {
            return BlobResponse {
                success: false,
                id: String::new(),
                content: String::new(),
                hash: String::new(),
            };
        }
        let path = self.resolve_blob_path(id);
        if !file_exists(path.as_str()) {
            return BlobResponse {
                success: false,
                id: String::new(),
                content: String::new(),
                hash: String::new(),
            };
        }
        match read_file(path.as_str()) {
            Ok(content) => BlobResponse {
                success: true,
                id: id.to_string(),
                content: content.clone(),
                hash: sha256::digest(content).to_string(),
            },
            Err(_) => BlobResponse {
                success: false,
                id: String::new(),
                content: String::new(),
                hash: String::new(),
            },
        }
    }

    pub fn update_blob(&self, id: &str, content: &str, old_hash: &str) -> BlobResponse {
        self.create_blob_dir();
        if !Self::is_valid_blob_id(id) {
            return BlobResponse {
                success: false,
                id: String::new(),
                content: String::new(),
                hash: String::new(),
            };
        }
        let path = self.resolve_blob_path(id);
        if !file_exists(path.as_str()) {
            return BlobResponse {
                success: false,
                id: String::new(),
                content: String::new(),
                hash: String::new(),
            };
        }
        match read_file(path.as_str()) {
            Ok(old_content) => {
                let hash = sha256::digest(old_content);
                if old_hash != hash {
                    return BlobResponse {
                        success: false,
                        id: String::new(),
                        content: String::new(),
                        hash: String::new(),
                    };
                }
                write_file(path.as_str(), content);
                BlobResponse {
                    success: true,
                    id: id.to_string(),
                    content: "".to_string(),
                    hash: sha256::digest(content).to_string(),
                }
            }
            Err(_) => {
                write_file(path.as_str(), content);
                BlobResponse {
                    success: true,
                    id: id.to_string(),
                    content: "".to_string(),
                    hash: sha256::digest(content).to_string(),
                }
            }
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
        true
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BlobResponse {
    pub success: bool,
    pub id: String,
    pub content: String,
    pub hash: String,
}
