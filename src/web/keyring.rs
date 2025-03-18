use super::{parse_request, Response};

pub fn handle_keyring(body: String) -> Response {
    let req = parse_request(body);
    match req.get_auth_user() {
        Ok(user) => match req.action.as_str() {
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
        },
        Err(err) => err,
    }
}
