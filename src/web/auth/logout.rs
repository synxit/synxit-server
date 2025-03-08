use super::Response;
use crate::{utils::u128_to_32_char_hex_string, web::Request};
use serde_json::json;

pub fn logout(req: Request) -> Response{
    let mut user = req.get_user();
    let session = user.get_session_by_id(req.data["session"].as_str().expect("No session provided"));
        user.delete_session_by_id(u128_to_32_char_hex_string(session.id).as_str());
        Response {
            success: true,
            data: json!({
                "message": "Logged out"
            })
        }
}
