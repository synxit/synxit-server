mod auth;
mod blob;
mod federation;
mod registration;

use crate::{
    logger::error::ERROR_USER_NOT_FOUND,
    synxit::{
        config::CONFIG,
        user::{MFAMethodPublic, User},
    },
    utils::current_time,
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
async fn auth_request(req_body: String) -> impl Responder {
    handle_auth(req_body).send()
}

#[post("/synxit/registration")]
async fn registration_request(req_body: String) -> impl Responder {
    handle_registration(req_body).send()
}

#[post("/synxit/blob")]
async fn blob_request(req_body: String) -> impl Responder {
    handle_blob(req_body).send()
}

#[post("/synxit/federation")]
async fn federation_request(req_body: String) -> impl Responder {
    handle_federation(req_body).await.send()
}

#[get("/synxit/status")]
async fn status() -> impl Responder {
    Response {
        success: true,
        data: json!({
            "message": "synxit server is running",
            "timestamp": current_time(),
            "synxit_version": env!("CARGO_PKG_VERSION"),
        }),
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub action: String,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub data: Value,
}

impl Response {
    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or(r#"{"success": false, "data": {}}"#.to_string())
    }

    pub fn send(&self) -> impl Responder {
        if self.success {
            HttpResponse::Ok()
                .append_header(("Access-Control-Allow-Origin", "*"))
                .body(self.to_string())
        } else {
            HttpResponse::BadRequest()
                .append_header(("Access-Control-Allow-Origin", "*"))
                .body(self.to_string())
        }
    }

    pub fn redirect(location: &str) -> impl Responder {
        HttpResponse::Found()
            .append_header(("Location", location))
            .finish()
    }

    pub fn error(message: &str) -> Self {
        Response {
            success: false,
            data: json!({ "error": message }),
        }
    }

    pub fn success(data: serde_json::Value) -> Self {
        Response {
            success: true,
            data,
        }
    }
}

impl Request {
    pub fn get_user(&self) -> Result<User, Response> {
        match self.data["username"].as_str() {
            None => Err(Response::error(ERROR_USER_NOT_FOUND)),
            Some(username) => match User::load(username) {
                Ok(user) => Ok(user),
                Err(err) => Err(Response::error(err.to_string().as_str())),
            },
        }
    }

    pub fn get_auth_user(&self) -> Result<User, Response> {
        match self.data["username"].as_str() {
            None => Err(Response::error(ERROR_USER_NOT_FOUND)),
            Some(username) => match User::load(username) {
                Ok(user) => {
                    if user.check_auth_by_id(self.data["session"].as_str().unwrap_or_default()) {
                        Ok(user)
                    } else {
                        Err(Response::error("Unauthorized"))
                    }
                }
                Err(err) => Err(Response::error(err.to_string().as_str())),
            },
        }
    }

    pub fn get_auth_completed_response(&self) -> Response {
        match self.get_user() {
            Ok(mut user) => match user.convert_auth_session_to_session(
                self.data["auth_session"].as_str().unwrap_or_default(),
            ) {
                Ok(session_id) => {
                    user.save();
                    Response {
                        success: true,
                        data: json!({
                            "username": user.username,
                            "status": "success",
                            "session": session_id,
                            "master_key": user.auth.encrypted.master_key,
                            "keyring": user.auth.encrypted.keyring,
                            "blob_map": user.auth.encrypted.blob_map
                        }),
                    }
                }
                Err(err) => match err {
                    "require_mfa" => {
                        let mut enabled_methods: Vec<MFAMethodPublic> = vec![];
                        let auth_session_id =
                            self.data["auth_session"].as_str().unwrap_or_default();

                        if let Ok(auth_session) = user.get_auth_session_by_id(auth_session_id) {
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

                        Response {
                            success: true,
                            data: json!({
                                "username": user.username,
                                "status": "require_mfa",
                                "methods": enabled_methods
                            }),
                        }
                    }
                    "require_password" => {
                        user.delete_auth_session_by_id(
                            self.data["auth_session"].as_str().unwrap_or_default(),
                        );
                        user.save();
                        Response {
                            success: false,
                            data: json!({
                                "status": "require_password"
                            }),
                        }
                    }
                    _ => Response::error("Unknown error"),
                },
            },
            Err(err) => err,
        }
    }
}

pub fn parse_request(req: String) -> Request {
    serde_json::from_str(req.as_str()).unwrap_or(Request {
        action: "".to_string(),
        data: json!({}),
    })
}
