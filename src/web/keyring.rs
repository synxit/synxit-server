use crate::synxit::user::User;

use super::{parse_request, Response};


pub fn handle_keyring(body: String) -> Response {
    let req = parse_request(body);
    let user = User::load(req.data["username"].as_str().expect("Failed to get username"));
    if req.check_auth() {
        match req.action.as_str() {
            "get" => Response {
                success: true,
                data: serde_json::json!({
                    "keyring": user.auth.encrypted.keyring
                }),
            },
            _ => Response {
                success: false,
                data: serde_json::json!({
                    "error": "Invalid action"
                }),
            },
        }
    } else {
        Response {
            success: false,
            data: serde_json::json!({
                "error": "Not authenticated"
            }),
        }
    }
}
