use serde_json::json;

use crate::{
    logger::error::{Error, ERROR_INVALID_ACTION, ERROR_INVALID_CREDENTIALS},
    synxit::user::{AuthSessionID, MFAMethodType, SessionID, UserHandle},
    utils::u128_to_32_char_hex_string,
};

use super::{Request, Response};

impl Request {
    pub fn userhandle(&self) -> Result<UserHandle, Error> {
        UserHandle::from_string(self.get_string("userhandle"))
    }

    pub fn response(&self) -> String {
        self.get_string("response")
    }

    pub fn auth_session(&self) -> AuthSessionID {
        self.get_string("auth_session").into()
    }

    pub fn session(&self) -> SessionID {
        self.get_string("session").into()
    }
}

pub fn handle_auth(req: Request) -> Response {
    match req.action() {
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
        "set_foreign_keyring" => foreign_keyring(req),
        "change_password" => change_password(req),
        _ => Response::error(ERROR_INVALID_ACTION),
    }
}

pub fn auth(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            if user.check_password_for_auth_session(req.auth_session(), &req.response()) {
                user.save();
                req.get_auth_completed_response()
            } else {
                Response::error(ERROR_INVALID_CREDENTIALS)
            }
        }
        Err(err) => err,
    }
}

pub fn prepare(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            let auth_session_id = user.create_auth_session();
            match user.get_auth_session_by_id(auth_session_id) {
                Ok(auth_session) => {
                    user.save();
                    Response::success(json!({
                        "auth_session": auth_session_id,
                        "challenge": u128_to_32_char_hex_string(auth_session.challenge),
                        "salt": user.auth.salt.to_string()
                    }))
                }
                Err(err) => Response::error(err.to_string().as_str()),
            }
        }
        Err(err) => err,
    }
}

pub fn logout(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            user.delete_session_by_id(req.session());
            if user.save() {
                Response::success(json!({}))
            } else {
                Response::error("Failed to logout")
            }
        }
        Err(err) => err,
    }
}

pub fn is_auth(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response::success(json!({
            "userhandle": user.userhandle.to_string()
        })),
        Err(err) => err,
    }
}

pub fn foreign_keyring(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            user.foreign_keyring = req.data["foreign_keyring"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if user.save() {
                Response::success(json!({}))
            } else {
                Response::error("Failed to save foreign key.")
            }
        }
        Err(err) => err,
    }
}

pub fn set_master_key(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            if let Some(master_key) = req.data["master_key"].as_str() {
                user.auth.encrypted.master_key = master_key.to_string();
                user.save();
                Response::success(json!({}))
            } else {
                Response::error("No master key provided")
            }
        }
        Err(err) => err,
    }
}

pub fn get_master_key(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response::success(json!({
            "master_key": user.auth.encrypted.master_key
        })),
        Err(err) => err,
    }
}

pub fn get_keyring(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response::success(json!({
            "keyring": user.auth.encrypted.keyring
        })),
        Err(err) => err,
    }
}

pub fn set_keyring(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            if let Some(keyring) = req.data["keyring"].as_str() {
                user.auth.encrypted.keyring = keyring.to_string();
                user.save();
                Response::success(json!({}))
            } else {
                Response::error("No keyring provided")
            }
        }
        Err(err) => err,
    }
}

pub fn change_password(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            let old_password = req.data["old_password"].as_str().unwrap_or_default();
            let new_password = req.data["password"].as_str().unwrap_or_default();
            let salt = req.data["salt"].as_str().unwrap_or_default();
            let master_key = req.data["master_key"].as_str().unwrap_or_default();
            if old_password == user.auth.hash {
                return Response::error(ERROR_INVALID_CREDENTIALS);
            }

            user.auth.hash = new_password.to_string();
            user.auth.salt = salt.to_string();
            user.auth.encrypted.master_key = master_key.to_string();

            user.save();
            Response::success(json!({}))
        }
        Err(err) => err,
    }
}

pub fn auth_mfa(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            if user.auth.mfa.enabled {
                if req.data.get("mfa_id").is_some() && req.data.get("mfa_code").is_some() {
                    if user.check_mfa(
                        req.data["mfa_id"].as_u64().unwrap_or(0) as u8,
                        req.data["mfa_code"].as_str().unwrap_or_default(),
                    ) {
                        user.auth_session_add_completed_mfa(
                            req.auth_session(),
                            req.data["mfa_id"].as_u64().unwrap_or(0) as u8,
                        );
                        if user.save() {
                            req.get_auth_completed_response()
                        } else {
                            Response::error("Failed to add MFA ID to session")
                        }
                    } else {
                        Response::error("Invalid MFA code")
                    }
                } else if req.data.get("mfa_recovery_code").is_some() {
                    let code = req.data["mfa_recovery_code"].as_str().unwrap_or_default();
                    if code.len() != 8 {
                        return Response::error("Invalid recovery code format");
                    }
                    if user.check_mfa_recovery_code(code) {
                        user.auth_session_add_completed_mfa(req.auth_session(), 255);
                        if user.save() {
                            req.get_auth_completed_response()
                        } else {
                            Response::error("Failed to add MFA ID to session")
                        }
                    } else {
                        Response::error("Invalid MFA recovery code")
                    }
                } else {
                    return Response::error("Missing MFA ID or code");
                }
            } else {
                Response::error("MFA is not enabled")
            }
        }
        Err(err) => err,
    }
}

pub fn add_mfa(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            let mfa_type = req.data["type"].as_str().unwrap_or_default();
            let mfa_name = req.data["name"].as_str().unwrap_or_default();
            if mfa_type == "totp" {
                if let Some(method) = user.create_mfa(MFAMethodType::TOTP, mfa_name.to_string()) {
                    if user.save() {
                        Response::success(serde_json::json!({
                            "method": method
                        }))
                    } else {
                        Response::error("Failed to save user with new MFA method")
                    }
                } else {
                    Response::error("Failed to create TOTP MFA method")
                }
            } else {
                Response::error("Invalid Method")
            }
        }
        Err(err) => err,
    }
}

pub fn list_mfa(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => {
            let methods: Vec<serde_json::Value> = user
                .auth
                .mfa
                .methods
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "name": m.name,
                        "type": m.r#type,
                        "enabled": m.enabled,
                    })
                })
                .collect();
            Response::success(
                serde_json::json!({ "enabled": user.auth.mfa.enabled, "methods": methods }),
            )
        }
        Err(err) => err,
    }
}

pub fn remove_mfa(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            let mfa_id = req.data["mfa_id"].as_u64().unwrap_or(0) as u8;
            if let Some(pos) = user.auth.mfa.methods.iter().position(|m| m.id == mfa_id) {
                user.auth.mfa.methods.remove(pos);
                if user.save() {
                    Response::success(serde_json::json!({}))
                } else {
                    Response::error("Failed to save user after removing MFA method")
                }
            } else {
                Response::error("MFA method not found")
            }
        }
        Err(err) => err,
    }
}

pub fn enable_mfa(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            user.auth.mfa.enabled = true;
            if user.save() {
                Response::success(serde_json::json!({}))
            } else {
                Response::error("Failed to enable MFA")
            }
        }
        Err(err) => err,
    }
}

pub fn disable_mfa(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => {
            user.auth.mfa.enabled = false;
            if user.save() {
                Response::success(serde_json::json!({}))
            } else {
                Response::error("Failed to disable MFA")
            }
        }
        Err(err) => err,
    }
}

pub fn new_recovery_codes(_req: Request) -> Response {
    match _req.get_auth_user() {
        Ok(mut user) => {
            user.generate_recovery_codes();
            if user.save() {
                Response::success(
                    serde_json::json!({ "recovery_codes": user.auth.mfa.recovery_codes }),
                )
            } else {
                Response::error("Failed to generate new recovery codes")
            }
        }
        Err(err) => err,
    }
}
