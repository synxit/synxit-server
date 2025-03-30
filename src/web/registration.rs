use serde_json::json;

use crate::synxit::user::User;

use super::{parse_request, Response};

pub fn handle_registration(body: String) -> Response {
    let req = parse_request(body);
    let username = req.data["username"].to_string();
    let password = req.data["password"].to_string();
    let salt = req.data["salt"].to_string();

    if User::user_exists(username.as_str()) {
        Response::error("Username already exists")
    } else {
        let user = User::new(username, password, salt);
        if user.save() {
            Response {
                success: true,
                data: json!({
                    "message": "Registration successful"
                }),
            }
        } else {
            Response::error("Unknown error")
        }
    }
}
