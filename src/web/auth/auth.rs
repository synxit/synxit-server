use crate::{logger::error::ERROR_INVALID_CREDENTIALS, web::{Request, Response}};
pub fn auth(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            if user.check_password_for_auth_session(
                req.data["auth_session"].as_str().unwrap_or_default(),
                req.data["response"].as_str().unwrap_or_default(),
            ) {
                user.save();
                req.get_auth_completed_response()
            }else {
                Response::error(ERROR_INVALID_CREDENTIALS)
            }
        }
        Err(err) => err,
    }
}
