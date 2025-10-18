use serde_json::json;

use crate::web::{Request, Response};

pub fn foreign_key(req: Request) -> Response {
    match req.get_auth_user() {
        Ok(user) => Response::success(json!({
            "foreign_key": user.foreign_key
        })),
        Err(err) => err,
    }
}
