use serde_json::json;

use crate::web::{Request, Response};

pub fn auth_mfa(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            if user.auth.mfa.enabled {
                if user.check_mfa(
                    req.data["mfa_id"].as_u64().unwrap_or(0) as u8,
                    req.data["mfa_code"].as_str().unwrap_or(""),
                ) {
                    user.auth_session_add_completed_mfa(
                        req.data["auth_session"].as_str().unwrap_or(""),
                        req.data["mfa_id"].as_u64().unwrap_or(0) as u8,
                    );
                    if user.save() {
                        Response {
                            success: true,
                            data: json!({
                                "auth_session": req.data["auth_session"].as_str().unwrap_or(""),
                                "mfa_id": req.data["mfa_id"].as_u64().unwrap_or(0),
                                "mfa_code": req.data["mfa_code"].as_str().unwrap_or("")
                            }),
                        }
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
