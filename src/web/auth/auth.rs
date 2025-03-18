use crate::web::{Request, Response};
pub fn auth(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            let correct_password = user.check_password_for_auth_session(
                req.data["auth_session"]
                    .as_str()
                    .expect("No auth session provided"),
                req.data["password"].as_str().expect("No password provided"),
            );
            user.save();
            if correct_password {
                if user.auth.mfa.enabled {
                    user.delete_auth_session_by_id(
                        req.data["auth_session"]
                            .as_str()
                            .expect("No auth session provided"),
                    );
                    user.save();
                    Response {
                        success: false,
                        data: serde_json::json!("not implemented"),
                    }
                } else {
                    match req.data["auth_session"].as_str() {
                        Some(auth_session) => {
                            match user.convert_auth_session_to_session(auth_session) {
                                Ok(session_id) => {
                                    user.save();
                                    Response {
                                        success: true,
                                        data: serde_json::json!({
                                            "session": session_id,
                                            "username": user.username,
                                            "master_key": user.auth.encrypted.master_key,
                                            "keyring": user.auth.encrypted.keyring,
                                            "blob_map": user.auth.encrypted.blob_map
                                        }),
                                    }
                                }
                                Err(err) => match err {
                                    "require_mfas" => Response {
                                        success: false,
                                        data: serde_json::json!("require_mfas"),
                                    },
                                    "require_password" => Response {
                                        success: false,
                                        data: serde_json::json!("require_email"),
                                    },
                                    _ => Response {
                                        success: false,
                                        data: serde_json::json!("Unknown error"),
                                    },
                                },
                            }
                        }
                        None => Response {
                            success: false,
                            data: serde_json::json!("No auth session provided"),
                        },
                    }
                }
            } else {
                Response {
                    success: false,
                    data: serde_json::json!({
                        "error": "Invalid password"
                    }),
                }
            }
        }
        Err(err) => err,
    }
}
