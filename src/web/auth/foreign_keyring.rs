use serde_json::json;

use crate::web::{Request, Response};

pub fn foreign_keyring(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            user.foreign_keyring = req.data["foreign_keyring"].as_str().unwrap_or("").to_string();
            if user.save() {
                Response::success(json!({}))
            } else {
                Response::error("Failed to save foreign key.")
            }
        }
        Err(err) => err,
    }
}
