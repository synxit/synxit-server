use crate::web::{Request, Response};
pub fn auth(req: Request) -> Response{
    let mut user = req.get_user();
    let correct_password = user.check_password_for_auth_session(req.data["auth_session"].as_str().expect("No auth session provided"), req.data["password"].as_str().expect("No password provided"));
    user.save();
    if correct_password {
        if user.auth.mfa.enabled {
            user.delete_auth_session_by_id(req.data["auth_session"].as_str().expect("No auth session provided"));
            user.save();
            Response {
                success: false,
                data: serde_json::json!("not implemented")
            }
        } else {
            let session = user.convert_auth_session_to_session(req.data["auth_session"].as_str().expect("No auth session provided"));
            user.save();
            Response {
                success: true,
                data: serde_json::json!({
                    "session": session,
                    "username": user.username,
                    "master_key": user.auth.encrypted.master_key,
                    "keyring": user.auth.encrypted.keyring,
                    "blob_map": user.auth.encrypted.blob_map
                })
            }
        }
    } else {
        Response {
            success: false,
            data: serde_json::json!({
                "error": "Invalid password"
            })
        }
    }
}
