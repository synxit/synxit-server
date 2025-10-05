use crate::web::{Request, Response};
pub fn auth(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            user.check_password_for_auth_session(
                req.data["auth_session"].as_str().unwrap_or_default(),
                req.data["password"].as_str().unwrap_or_default(),
            );
            user.save();
            req.get_auth_completed_response()
        }
        Err(err) => err,
    }
}
