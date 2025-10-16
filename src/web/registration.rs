use log::info;
use serde_json::json;

use crate::{
    logger::error::ERROR_REGISTRATION_DISABLED,
    synxit::{config::get_config, user::User},
};

use super::{parse_request, Response};

pub fn handle_registration(body: String) -> Response {
    if !get_config().auth.registration_enabled {
        return Response::error(ERROR_REGISTRATION_DISABLED);
    }
    let req = parse_request(body);
    let username = req.data["username"].as_str().unwrap_or_default();
    let password = req.data["password"].as_str().unwrap_or_default();
    let salt = req.data["salt"].as_str().unwrap_or_default();
    if User::user_exists(username) {
        Response::error("Username already exists")
    } else {
        let user = User::new(username, password, salt);
        if user.save() {
            info!("New user registered: {}", user.username);
            Response::success(json!({
                "username": user.username,
            }))
        } else {
            Response::error("Unknown error")
        }
    }
}
