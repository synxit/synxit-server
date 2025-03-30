use crate::{
    utils::u128_to_32_char_hex_string,
    web::{Request, Response},
};
use serde_json::json;

pub fn prepare(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            let auth_session_id = user.create_auth_session();
            match user.get_auth_session_by_id(auth_session_id.as_str()) {
                Ok(auth_session) => {
                    user.save();
                    Response {
                        success: true,
                        data: json!({
                            "auth_session": auth_session_id,
                            "challenge": u128_to_32_char_hex_string(auth_session.challenge),
                            "salt": user.auth.salt.to_string()
                        }),
                    }
                }
                Err(err) => Response::error(err.to_string().as_str()),
            }
        }
        Err(err) => err,
    }
}
