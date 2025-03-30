use super::Response;
use crate::web::Request;
use serde_json::json;

pub fn is_auth(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response {
            success: true,
            data: json!({
                "username": user.username
            }),
        },
        Err(err) => err,
    }
}
