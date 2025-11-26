use crate::{
    synxit::user::MFAMethodType,
    web::{Request, Response},
};

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
                            req.data["auth_session"].as_str().unwrap_or_default(),
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
                        user.auth_session_add_completed_mfa(
                            req.data["auth_session"].as_str().unwrap_or_default(),
                            255,
                        );
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
