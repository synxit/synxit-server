use super::{super::Request, Response};
use serde_json::json;

pub fn set_master_key(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            user.auth.encrypted.master_key = req.data["master_key"]
                .as_str()
                .expect("No master key provided")
                .to_string();
            user.save();
            Response {
                success: true,
                data: json!({
                    "message": "Master key set"
                }),
            }
        }
        Err(err) => err,
    }
}

pub fn get_master_key(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response {
            success: true,
            data: json!({
                "master_key": user.auth.encrypted.master_key
            }),
        },
        Err(err) => err,
    }
}

pub fn get_keyring(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response {
            success: true,
            data: json!({
                "keyring": user.auth.encrypted.keyring
            }),
        },
        Err(err) => err,
    }
}

pub fn set_keyring(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            user.auth.encrypted.keyring = req.data["keyring"]
                .as_str()
                .expect("No keyring provided")
                .to_string();
            user.save();
            Response {
                success: true,
                data: json!({
                    "message": "Keyring set"
                }),
            }
        }
        Err(err) => err,
    }
}

pub fn get_blob_map(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response {
            success: true,
            data: json!({
                "blob_map": user.auth.encrypted.blob_map
            }),
        },
        Err(err) => err,
    }
}

pub fn set_blob_map(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            user.auth.encrypted.blob_map = req.data["blob_map"]
                .as_str()
                .expect("No blob map provided")
                .to_string();
            user.save();
            Response {
                success: true,
                data: json!({
                    "message": "Blob map set"
                }),
            }
        }
        Err(err) => err,
    }
}
