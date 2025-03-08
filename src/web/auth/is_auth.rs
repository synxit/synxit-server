use super::Response;
use crate::web::Request;
use serde_json::json;

pub fn is_auth(req: Request) -> Response{
    let user = req.get_user();
    if req.check_auth() {
        Response {
            success: true,
            data: json!({
                "username": user.username
            })
        }
    }else{
        Response {
            success: false,
            data: json!({
                "error": "Invalid session"
            })
        }
    }
}
