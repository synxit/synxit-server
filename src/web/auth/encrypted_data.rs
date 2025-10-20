use crate::logger::error::ERROR_INVALID_CREDENTIALS;

use super::{super::Request, Response};
use serde_json::json;

pub fn set_master_key(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            if let Some(master_key) = req.data["master_key"].as_str() {
                user.auth.encrypted.master_key = master_key.to_string();
                user.save();
                Response::success(json!({}))
            } else {
                Response::error("No master key provided")
            }
        }
        Err(err) => err,
    }
}

pub fn get_master_key(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response::success(json!({
            "master_key": user.auth.encrypted.master_key
        })),
        Err(err) => err,
    }
}

pub fn get_keyring(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response::success(json!({
            "keyring": user.auth.encrypted.keyring
        })),
        Err(err) => err,
    }
}

pub fn set_keyring(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            if let Some(keyring) = req.data["keyring"].as_str() {
                user.auth.encrypted.keyring = keyring.to_string();
                user.save();
                Response::success(json!({}))
            } else {
                Response::error("No keyring provided")
            }
        }
        Err(err) => err,
    }
}

pub fn change_password(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            let old_password = req.data["old_password"].as_str().unwrap_or_default();
            let new_password = req.data["password"].as_str().unwrap_or_default();
            let salt = req.data["salt"].as_str().unwrap_or_default();
            let master_key = req.data["master_key"].as_str().unwrap_or_default();
            if old_password == user.auth.hash {
                return Response::error(ERROR_INVALID_CREDENTIALS);
            }

            user.auth.hash = new_password.to_string();
            user.auth.salt = salt.to_string();
            user.auth.encrypted.master_key = master_key.to_string();

            user.save();
            Response::success(json!({}))
        }
        Err(err) => err,
    }
}
