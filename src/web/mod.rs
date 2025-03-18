mod auth;
mod blob;
mod federation;
mod keyring;

use crate::{
    synxit::{config::CONFIG, user::User},
    utils::current_time,
};
use actix_web::{get, post, routes, web::PayloadConfig, App, HttpResponse, HttpServer, Responder};
use auth::handle_auth;
use blob::handle_blob;
use keyring::handle_keyring;
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

#[post("/synxit/blob")]
async fn blob_request(req_body: String) -> impl Responder {
    handle_blob(req_body).send()
}

#[post("/synxit/keyring")]
async fn keyring_request(req_body: String) -> impl Responder {
    handle_keyring(req_body).send()
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
#[options("/synxit/blob")]
#[options("/synxit/keyring")]
async fn options_request() -> impl Responder {
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Access-Control-Allow-Methods", "POST, OPTIONS"))
        .append_header(("Access-Control-Allow-Headers", "Content-Type"))
        .finish()
}

pub async fn start_server() -> std::io::Result<()> {
    let config = CONFIG.get().unwrap();
    HttpServer::new(|| {
        App::new()
            .app_data(PayloadConfig::new(1024 * 1024 * 1024 * 4))
            .service(redirect)
            .service(auth_request)
            .service(blob_request)
            .service(keyring_request)
            .service(options_request)
            .service(status)
    })
    .bind((config.network.host.to_string(), config.network.port))?
    .run()
    .await
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
        serde_json::to_string(&self).expect("Failed to serialize response")
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
}

impl Request {
    pub fn get_user(&self) -> Result<User, Response> {
        match self.data["username"].as_str() {
            None => Err(Response {
                success: false,
                data: serde_json::json!({ "error": "Username not provided" }),
            }),
            Some(username) => match User::load(username) {
                Ok(user) => Ok(user),
                Err(err) => Err(Response {
                    success: false,
                    data: serde_json::json!({ "error": err.to_string() }),
                }),
            },
        }
    }

    pub fn get_auth_user(&self) -> Result<User, Response> {
        match self.data["username"].as_str() {
            None => Err(Response {
                success: false,
                data: serde_json::json!({ "error": "Username not provided" }),
            }),
            Some(username) => match User::load(username) {
                Ok(user) => {
                    if user.check_auth_by_id(self.data["session"].as_str().unwrap()) {
                        Ok(user)
                    } else {
                        Err(Response {
                            success: false,
                            data: serde_json::json!({ "error": "Unauthorized" }),
                        })
                    }
                }
                Err(err) => Err(Response {
                    success: false,
                    data: serde_json::json!({ "error": err.to_string() }),
                }),
            },
        }
    }
}

pub fn parse_request(req: String) -> Request {
    serde_json::from_str(req.as_str()).unwrap_or(Request {
        action: "".to_string(),
        data: serde_json::json!({}),
    })
}
