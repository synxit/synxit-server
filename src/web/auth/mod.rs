use auth::auth;
use is_auth::is_auth;
use logout::logout;
use prepare::prepare;

use super::{parse_request, Response};

mod auth;
mod encrypted_data;
mod is_auth;
mod logout;
mod prepare;

pub fn handle_auth(body: String) -> Response {
    let req = parse_request(body);
    match req.action.as_str() {
        "prepare" => prepare(req),
        "auth" => auth(req),
        "is_auth" => is_auth(req),
        "logout" => logout(req),
        "get_master_key" => encrypted_data::get_master_key(req),
        "set_master_key" => encrypted_data::set_master_key(req),
        "get_keyring" => encrypted_data::get_keyring(req),
        "set_keyring" => encrypted_data::set_keyring(req),
        "get_blob_map" => encrypted_data::get_blob_map(req),
        "set_blob_map" => encrypted_data::set_blob_map(req),
        _ => Response {
            success: false,
            data: serde_json::json!({
                "error": "Invalid action"
            }),
        },
    }
}
