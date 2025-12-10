use super::{Request, Response};
use crate::{
    logger::error::{
        ERROR_BLOB_NOT_FOUND, ERROR_INVALID_ACTION, ERROR_INVALID_JSON, ERROR_NO_WRITE_ACCESS,
        ERROR_REMOTE_ERROR, ERROR_SHARE_NOT_FOUND, ERROR_USER_NOT_FOUND,
    },
    synxit::{
        config::get_config,
        user::{
            blob::{BlobID, Share, ShareID, ShareSecret},
            User, UserHandle,
        },
    },
};
use serde_json::json;

impl Request {
    pub fn share_id(&self) -> ShareID {
        self.get_string("share_id").into()
    }

    pub fn share_secret(&self) -> ShareSecret {
        self.get_string("share_secret").into()
    }

    pub fn share_user(&self) -> Result<UserHandle, Response> {
        UserHandle::from_string(self.get_string("share_user"))
            .map_err(|_| Response::error(ERROR_USER_NOT_FOUND))
    }
}

/// Handles federation-related actions such as proxy, blobs, read, update, delete, etc.
pub async fn handle_federation(req: Request) -> Response {
    if let Err(response) = validate_federation_config(&req) {
        return response;
    }

    match req.action() {
        "proxy" => handle_proxy_action(&req).await,
        "blobs" => handle_blobs_action(&req),
        "read" => handle_read_action(&req),
        "update" => handle_update_action(&req),
        "delete" => handle_delete_action(&req),
        "create" => handle_create_action(&req),
        "foreign_key" => handle_foreign_key_action(&req),
        _ => Response::error(ERROR_INVALID_ACTION),
    }
}

/// Validates the federation configuration.
fn validate_federation_config(req: &Request) -> Result<(), Response> {
    let config = get_config();
    let share_user = req.data["share_user"].as_str().unwrap_or_default();
    let server = User::resolve_user(share_user)
        .unwrap_or((String::new(), String::new()))
        .1;

    if !config.federation.enabled
        || (config.federation.whitelist.enabled
            && !config.federation.whitelist.hosts.contains(&server))
        || (config.federation.blacklist.enabled
            && config.federation.blacklist.hosts.contains(&server))
    {
        return Err(Response::error(ERROR_REMOTE_ERROR));
    }

    Ok(())
}

/// Handles the "proxy" action.
async fn handle_proxy_action(req: &Request) -> Response {
    let share_user = match req.share_user() {
        Ok(user) => user,
        Err(response) => return response,
    };

    let url = resolve_federation_url(&share_user);
    let action = req.data["action"].as_str().unwrap_or_default();
    let request_body = build_proxy_request_body(action, &share_user, req);

    match post_request(url, &request_body).await {
        Ok(success) => {
            serde_json::from_value(success).unwrap_or_else(|_| Response::error(ERROR_INVALID_JSON))
        }
        Err(_) => Response::error(ERROR_REMOTE_ERROR),
    }
}

/// Resolves the federation URL for the given user.
fn resolve_federation_url(share_user: &UserHandle) -> String {
    format!("http://{}:8400/synxit/federation", share_user.get_server())
}

/// Builds the request body for the proxy action.
fn build_proxy_request_body(
    action: &str,
    share_user: &UserHandle,
    req: &Request,
) -> serde_json::Value {
    let mut data = json!({
        "id": req.share_id(),
        "share_user": share_user,
        "secret": req.share_secret(),
    });

    if matches!(action, "update" | "create") {
        data["content"] = req.data["content"].clone();
    }

    if matches!(action, "update" | "delete" | "read") {
        data["blob"] = req.data["blob"].clone();
    }

    if action == "update" {
        data["hash"] = req.data["hash"].clone();
    }

    json!({ "action": action, "data": data })
}

/// Handles the "blobs" action.
fn handle_blobs_action(req: &Request) -> Response {
    match validate_user_and_share(req) {
        Ok(share) => Response::success(json!({
            "blobs": share.1.blobs,
            "write_access": share.1.write,
        })),
        Err(response) => response,
    }
}

/// Handles the "read" action.
fn handle_read_action(req: &Request) -> Response {
    let blob_id = req.blob_id();
    match validate_user_and_blob(req, blob_id, false) {
        Ok(user) => match user.read_blob(blob_id) {
            Ok(result) => Response::success(json!({
                "content": result.0,
                "hash": result.1,
            })),
            Err(e) => Response::error(e.to_string().as_str()),
        },
        Err(response) => response,
    }
}

/// Handles the "update" action.
fn handle_update_action(req: &Request) -> Response {
    let blob_id = req.blob_id();
    let content = req.content();
    let old_hash = req.blob_hash();

    match validate_user_and_blob(req, blob_id, true) {
        Ok(user) => match user.update_blob(blob_id, content, old_hash) {
            Ok(hash) => Response::success(json!({ "hash": hash })),
            Err(e) => Response::error(e.to_string().as_str()),
        },
        Err(response) => response,
    }
}

/// Handles the "delete" action.
fn handle_delete_action(req: &Request) -> Response {
    let blob_id = req.blob_id();
    match validate_user_and_blob(req, blob_id, true) {
        Ok(user) => {
            if user.delete_blob(blob_id) {
                Response::success(json!({}))
            } else {
                Response::error(ERROR_BLOB_NOT_FOUND)
            }
        }
        Err(response) => response,
    }
}

/// Handles the "create" action.
fn handle_create_action(req: &Request) -> Response {
    match validate_user_and_share(req) {
        Ok(share) => {
            if !share.1.write {
                return Response::error(ERROR_NO_WRITE_ACCESS);
            }
            match share.0.create_blob(req.content()) {
                Ok((blob_id, hash)) => match share.0.add_blob_to_share(share.1.id, blob_id) {
                    Ok(_) => Response::success(json!({ "id": blob_id, "hash": hash })),
                    Err(_) => Response::error("message"),
                },
                Err(e) => Response::error(e.to_string().as_str()),
            }
        }
        Err(response) => response,
    }
}

/// Handles the "foreign_key" action.
fn handle_foreign_key_action(req: &Request) -> Response {
    match req.share_user() {
        Ok(user) => match User::load(user) {
            Ok(user) => Response::success(json!({ "foreign_key": user.foreign_keyring })),
            Err(err) => Response::error(err.to_string().as_str()),
        },
        Err(response) => response,
    }
}

/// Validates the user and share access.
fn validate_user_and_share(req: &Request) -> Result<(User, Share), Response> {
    let share_user = req.share_user()?;
    let user = User::load(share_user).map_err(|_| Response::error(ERROR_USER_NOT_FOUND))?;
    match user.validate_share_access(req.share_id(), req.share_secret()) {
        Ok(share) => Ok((user, share)),
        Err(_) => Err(Response::error(ERROR_SHARE_NOT_FOUND)),
    }
}

/// Validates the user and blob access.
fn validate_user_and_blob(
    req: &Request,
    blob_id: BlobID,
    write_access: bool,
) -> Result<User, Response> {
    let share_user = req.share_user()?;
    let user = User::load(share_user).map_err(|_| Response::error(ERROR_USER_NOT_FOUND))?;
    user.validate_blob_access(req.share_id(), req.share_secret(), blob_id, write_access)
        .map_err(|response| Response::error(response.to_string().as_str()))?;
    Ok(user)
}

/// Sends a POST request to the given URL with the provided JSON body.
async fn post_request(
    url: String,
    json_body: &serde_json::Value,
) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();
    client.post(url).json(json_body).send().await?.json().await
}
