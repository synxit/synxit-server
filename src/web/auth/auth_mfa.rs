use crate::{synxit::user::MFAMethodType, web::{Request, Response}};

pub fn auth_mfa(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            if user.auth.mfa.enabled {
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
            } else {
                Response::error("MFA is not enabled")
            }
        }
        Err(err) => err,
    }
}

pub fn add_mfa(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            let mfa_type = req.data["type"].as_str().unwrap_or_default();
            let mfa_name = req.data["name"].as_str().unwrap_or_default();
            if mfa_type == "totp" {
                let res = Response::success(serde_json::json!({
                    "method": user.create_mfa(MFAMethodType::TOTP, mfa_name.to_string())
                }));
                if user.save() {
                    res
                } else {
                    Response::error("Failed to save user with new MFA method")
                }
            }else{
                Response::error("Invalid Method")
            }
        }
        Err(err) => err,
    }
}
