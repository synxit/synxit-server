use super::Response;
use crate::{synxit::user::User, web::Request};
use serde_json::json;

pub fn set_master_key(req: Request) -> Response{
    let mut user = User::load(req.data["username"].as_str().expect("No username provided"));
    if req.check_auth() {
        user.auth.encrypted.master_key = req.data["master_key"].as_str().expect("No master key provided").to_string();
        user.save();
        Response {
            success: true,
            data: json!({
                "message": "Master key set"
            })
        }
    }else{
        Response {
            success: false,
            data: json!({
                "error": "Invalid session"
            })
        }
    }
}

pub fn get_master_key(req: Request) -> Response{
    let user = User::load(req.data["username"].as_str().expect("No username provided"));
    if req.check_auth() {
        Response {
            success: true,
            data: json!({
                "master_key": user.auth.encrypted.master_key
            })
        }
    }else{
        Response {
            success: false,
            data: json!({
                "error": "Invalid session"
            })
        }
    }
}

pub fn get_keyring(req: Request) -> Response{
    let user = User::load(req.data["username"].as_str().expect("No username provided"));
    if req.check_auth() {
        Response {
            success: true,
            data: json!({
                "keyring": user.auth.encrypted.keyring
            })
        }
    }else{
        Response {
            success: false,
            data: json!({
                "error": "Invalid session"
            })
        }
    }
}

pub fn set_keyring(req: Request) -> Response{
    let mut user = User::load(req.data["username"].as_str().expect("No username provided"));
    if req.check_auth() {
        user.auth.encrypted.keyring = req.data["keyring"].as_str().expect("No keyring provided").to_string();
        user.save();
        Response {
            success: true,
            data: json!({
                "message": "Keyring set"
            })
        }
    }else{
        Response {
            success: false,
            data: json!({
                "error": "Invalid session"
            })
        }
    }
}

pub fn get_blob_map(req: Request) -> Response{
    let user = User::load(req.data["username"].as_str().expect("No username provided"));
    if req.check_auth() {
        Response {
            success: true,
            data: json!({
                "blob_map": user.auth.encrypted.blob_map
            })
        }
    }else{
        Response {
            success: false,
            data: json!({
                "error": "Invalid session"
            })
        }
    }
}

pub fn set_blob_map(req: Request) -> Response{
    let mut user = User::load(req.data["username"].as_str().expect("No username provided"));
    if req.check_auth() {
        user.auth.encrypted.blob_map = req.data["blob_map"].as_str().expect("No blob map provided").to_string();
        user.save();
        Response {
            success: true,
            data: json!({
                "message": "Blob map set"
            })
        }
    }else{
        Response {
            success: false,
            data: json!({
                "error": "Invalid session"
            })
        }
    }
}
