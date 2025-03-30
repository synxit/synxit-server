use log::{debug, warn};

use crate::web::{Request, Response};
pub fn auth(req: Request) -> Response {
    match req.get_user() {
        Ok(mut user) => {
            if user.check_password_for_auth_session(
                req.data["auth_session"].as_str().unwrap_or(""),
                req.data["password"].as_str().unwrap_or(""),
            ) {
                debug!("User authenticated successfully");
            } else {
                warn!("User authentication failed");
            }
            user.save();
            req.get_auth_completed_response()
        }
        Err(err) => err,
    }
}
