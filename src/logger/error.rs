pub const ERROR_INVALID_ACTION: &str = "INVALID_ACTION";
pub const ERROR_WRONG_SECRET: &str = "WRONG_SECRET";
pub const ERROR_USER_NOT_FOUND: &str = "USER_NOT_FOUND";
pub const ERROR_BLOB_NOT_FOUND: &str = "BLOB_NOT_FOUND";
pub const ERROR_BLOB_NOT_IN_SHARE: &str = "BLOB_IS_NOT_IN_SHARE";
pub const ERROR_NO_WRITE_ACCESS: &str = "NO_WRITE_ACCESS";
pub const ERROR_REMOTE_ERROR: &str = "REMOTE_ERROR";
pub const ERROR_INVALID_JSON: &str = "INVALID_JSON";
pub const ERROR_QUOTA_EXCEEDED: &str = "QUOTA_EXCEEDED";
pub const ERROR_BLOB_HASH_NOT_MATCH: &str = "BLOB_HASH_NOT_MATCH";
pub const ERROR_SHARE_NOT_FOUND: &str = "SHARE_NOT_FOUND";
pub const ERROR_INVALID_CREDENTIALS: &str = "INVALID_CREDENTIALS";

pub const ERROR_INVALID_SESSION: &str = "INVALID_SESSION";
pub const ERROR_REGISTRATION_DISABLED: &str = "REGISTRATION_DISABLED";

pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
