use super::Response;
use crate::web::Request;
use serde_json::json;

pub fn logout(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(mut user) => match req.data["session"].as_str() {
            Some(session_id) => {
                if user.delete_session_by_id(session_id) {
                    user.save();
                    Response {
                        success: true,
                        data: json!({
                            "message": "Logged out"
                        }),
                    }
                } else {
                    Response {
                        success: false,
                        data: json!({
                            "message": "Failed to log out"
                        }),
                    }
                }
            }
            None => Response {
                success: false,
                data: json!({
                    "message": "No session provided"
                }),
            },
        },
        Err(err) => err,
    }
}
