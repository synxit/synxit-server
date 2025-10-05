use auth::auth;
use auth_mfa::{add_mfa, auth_mfa};
use is_auth::is_auth;
use logout::logout;
use prepare::prepare;

use crate::logger::error::ERROR_INVALID_ACTION;

use super::{parse_request, Response};

mod auth;
mod auth_mfa;
mod encrypted_data;
mod is_auth;
mod logout;
mod prepare;

pub fn handle_auth(body: String) -> Response {
    let req = parse_request(body);
    match req.action.as_str() {
        "prepare" => prepare(req),
        "auth_mfa" => auth_mfa(req),
        "auth" => auth(req),
        "is_auth" => is_auth(req),
        "logout" => logout(req),
        "add_mfa" => add_mfa(req),
        "get_master_key" => encrypted_data::get_master_key(req),
        "set_master_key" => encrypted_data::set_master_key(req),
        "get_keyring" => encrypted_data::get_keyring(req),
        "set_keyring" => encrypted_data::set_keyring(req),
        "get_blob_map" => encrypted_data::get_blob_map(req),
        "set_blob_map" => encrypted_data::set_blob_map(req),
        _ => Response::error(ERROR_INVALID_ACTION),
    }
}
