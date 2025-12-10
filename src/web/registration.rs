use log::info;
use serde_json::json;

use crate::{
    logger::error::ERROR_REGISTRATION_DISABLED,
    synxit::{config::get_config, user::User},
};

use super::{Request, Response};

pub fn handle_registration(req: Request) -> Response {
    if !get_config().auth.registration_enabled {
        return Response::error(ERROR_REGISTRATION_DISABLED);
    }
    let userhandle = match req.userhandle() {
        Ok(userhandle) => userhandle,
        Err(_) => return Response::error("Invalid username"),
    };
    let password = req.data["password"].as_str().unwrap_or_default();
    let salt = req.data["salt"].as_str().unwrap_or_default();
    if User::user_exists(userhandle.to_owned()) {
        Response::error("Username already exists")
    } else {
        let user = User::new(userhandle, password, salt);
        if user.save() {
            info!("New user registered: {}", user.userhandle.to_string());
            Response::success(json!({
                "username": user.userhandle,
            }))
        } else {
            Response::error("Unknown error")
        }
    }
}
