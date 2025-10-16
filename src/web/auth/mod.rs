use auth::auth;
use auth_mfa::{
    add_mfa, auth_mfa, disable_mfa, enable_mfa, list_mfa, new_recovery_codes, remove_mfa,
};
use encrypted_data::{get_keyring, get_master_key, set_keyring, set_master_key};
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
        "enable_mfa" => enable_mfa(req),
        "disable_mfa" => disable_mfa(req),
        "list_mfa" => list_mfa(req),
        "remove_mfa" => remove_mfa(req),
        "get_master_key" => get_master_key(req),
        "set_master_key" => set_master_key(req),
        "get_keyring" => get_keyring(req),
        "set_keyring" => set_keyring(req),
        "new_recovery_codes" => new_recovery_codes(req),
        _ => Response::error(ERROR_INVALID_ACTION),
    }
}
