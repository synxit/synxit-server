mod auth;
mod blob;
mod federation;
mod registration;

use std::fmt::Display;

use crate::{
    logger::error::{ERROR_UNAUTHORIZED, ERROR_USER_NOT_FOUND},
    utils::{as_str, current_time},
    {
        config::CONFIG,
        user::{MFAMethodPublic, User},
    },
};
use actix_web::{get, post, routes, web::PayloadConfig, App, HttpResponse, HttpServer, Responder};
use auth::handle_auth;
use blob::handle_blob;
use federation::handle_federation;
use registration::handle_registration;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[get("/")]
async fn redirect() -> impl Responder {
    Response::redirect("https://app.synxit.de")
}

#[post("/synxit/auth")]
async fn auth_request(body: String) -> impl Responder {
    handle_auth(Request::parse(body)).send()
}

#[post("/synxit/registration")]
async fn registration_request(body: String) -> impl Responder {
    handle_registration(Request::parse(body)).send()
}

#[post("/synxit/blob")]
async fn blob_request(body: String) -> impl Responder {
    handle_blob(Request::parse(body)).send()
}

#[post("/synxit/federation")]
async fn federation_request(body: String) -> impl Responder {
    handle_federation(Request::parse(body)).await.send()
}

#[get("/synxit/status")]
async fn status() -> impl Responder {
    Response::success(json!({
        "message": "synxit server is running",
        "timestamp": current_time(),
        "synxit_version": env!("CARGO_PKG_VERSION"),
    }))
    .send()
}

#[routes]
#[options("/synxit/auth")]
#[options("/synxit/registration")]
#[options("/synxit/blob")]
#[options("/synxit/federation")]
async fn options_request() -> impl Responder {
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Access-Control-Allow-Methods", "POST, OPTIONS"))
        .append_header(("Access-Control-Allow-Headers", "Content-Type"))
        .finish()
}

pub async fn start_server() {
    let config = CONFIG.get().unwrap();
    match HttpServer::new(|| {
        App::new()
            .app_data(PayloadConfig::new(1024 * 1024 * 1024 * 4))
            .service(redirect)
            .service(auth_request)
            .service(registration_request)
            .service(blob_request)
            .service(options_request)
            .service(federation_request)
            .service(status)
    })
    .bind((config.network.host.to_string(), config.network.port))
    {
        Ok(server) => match server.run().await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Cannot start server: {}", err);
            }
        },
        Err(err) => {
            log::error!("Cannot bind address: {}", err);
        }
    };
}

#[derive(Serialize, Deserialize)]
struct Request {
    action: String,
    data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response(Result<Value, String>);

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Ok(data) => write!(f, "{}", json!({ "success": true, "data": data })),
            Err(err) => write!(
                f,
                "{}",
                json!({ "success": false, "data": { "error": err} })
            ),
        }
    }
}

impl Response {
    pub fn send(&self) -> impl Responder {
        match &self.0 {
            Ok(_) => HttpResponse::Ok()
                .append_header(("Access-Control-Allow-Origin", "*"))
                .append_header(("Content-Type", "application/json"))
                .body(self.to_string()),
            Err(err) if err == ERROR_UNAUTHORIZED => HttpResponse::Unauthorized()
                .append_header(("Access-Control-Allow-Origin", "*"))
                .append_header(("Content-Type", "application/json"))
                .body(self.to_string()),
            Err(_) => HttpResponse::BadRequest()
                .append_header(("Access-Control-Allow-Origin", "*"))
                .append_header(("Content-Type", "application/json"))
                .body(self.to_string()),
        }
    }

    pub fn redirect(location: &str) -> impl Responder {
        HttpResponse::Found()
            .append_header(("Location", location))
            .finish()
    }

    pub fn error(message: &str) -> Self {
        Response(Err(message.to_string()))
    }

    pub fn success(data: serde_json::Value) -> Self {
        Response(Ok(data))
    }
}

impl Request {
    pub fn parse(req: String) -> Self {
        serde_json::from_str(req.as_str()).unwrap_or(Request {
            action: "".to_string(),
            data: json!({}),
        })
    }

    pub fn get_user(&self) -> Result<User, Response> {
        match self.userhandle() {
            Err(_) => Err(Response::error(ERROR_USER_NOT_FOUND)),
            Ok(userhandle) => match User::load(userhandle) {
                Ok(user) => Ok(user),
                Err(err) => Err(Response::error(err.to_string().as_str())),
            },
        }
    }

    pub fn get_auth_user(&self) -> Result<User, Response> {
        match self.userhandle() {
            Err(_) => Err(Response::error(ERROR_USER_NOT_FOUND)),
            Ok(userhandle) => match User::load(userhandle) {
                Ok(user) => {
                    if user.check_auth_by_id(self.session()) {
                        Ok(user)
                    } else {
                        Err(Response::error(ERROR_UNAUTHORIZED))
                    }
                }
                Err(err) => Err(Response::error(err.to_string().as_str())),
            },
        }
    }

    pub fn get_auth_completed_response(&self) -> Response {
        match self.get_user() {
            Ok(mut user) => match user.convert_auth_session_to_session(self.auth_session()) {
                Ok(session_id) => {
                    user.save();
                    Response::success(json!({
                        "username": user.userhandle,
                        "status": "success",
                        "session": session_id,
                        "master_key": user.auth.encrypted.master_key,
                        "keyring": user.auth.encrypted.keyring,
                        "blob_map": user.auth.encrypted.blob_map
                    }))
                }
                Err(err) => match err {
                    "require_mfa" => {
                        let mut enabled_methods: Vec<MFAMethodPublic> = vec![];

                        if let Ok(auth_session) = user.get_auth_session_by_id(self.auth_session()) {
                            for method in &user.auth.mfa.methods {
                                if method.enabled
                                    && !auth_session.completed_mfa.contains(&method.id)
                                {
                                    enabled_methods.push(MFAMethodPublic {
                                        id: method.id,
                                        name: method.name.clone(),
                                        r#type: method.r#type.clone(),
                                    });
                                }
                            }
                        }

                        Response::success(json!({
                            "username": user.userhandle,
                            "status": "require_mfa",
                            "methods": enabled_methods
                        }))
                    }
                    "require_password" => {
                        user.delete_auth_session_by_id(self.auth_session());
                        user.save();
                        Response::success(json!({
                            "status": "require_password"
                        }))
                    }
                    _ => Response::error("Unknown error"),
                },
            },
            Err(err) => err,
        }
    }

    pub fn action(&self) -> &str {
        self.action.as_str()
    }

    pub fn get_str(&self, field: &str) -> &str {
        as_str(&self.data[field])
    }

    pub fn get_string(&self, field: &str) -> String {
        self.get_str(field).to_string()
    }
}
