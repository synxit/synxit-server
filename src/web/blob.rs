use super::{Request, Response};
use crate::{
    logger::error::ERROR_INVALID_ACTION,
    synxit::user::blob::{Base64, BlobHash, BlobID},
};
use serde_json::json;

impl Request {
    pub fn content(&self) -> Base64 {
        self.get_string("content").into()
    }

    pub fn blob_id(&self) -> BlobID {
        self.get_string("blob_id").into()
    }

    pub fn blob_hash(&self) -> BlobHash {
        self.get_string("blob_hash").into()
    }
}

/// Handles blob-related actions such as create, read, update, delete, etc.
pub fn handle_blob(req: Request) -> Response {
    // Authenticate user
    let mut user = match req.get_auth_user() {
        Ok(user) => user,
        Err(err) => return err,
    };

    // Dispatch action
    match req.action() {
        "create" => handle_create_blob(&user, &req),
        "read" => handle_read_blob(&user, &req),
        "update" => handle_update_blob(&user, &req),
        "delete" => handle_delete_blob(&user, &req),
        "hash" => handle_blob_hash(&user, &req),
        "set_blob_map" => handle_set_blob_map(&mut user, &req),
        "get_blob_map" => handle_get_blob_map(&user),
        "get_quota" => handle_get_quota(&user),
        _ => Response::error(ERROR_INVALID_ACTION),
    }
}

/// Handles the creation of a new blob.
fn handle_create_blob(user: &crate::synxit::user::User, req: &super::Request) -> Response {
    match user.create_blob(req.content()) {
        Ok(blob) => Response::success(json!({
            "id": blob.0,
            "hash": blob.1
        })),
        Err(e) => Response::error(e.to_string().as_str()),
    }
}

/// Handles reading an existing blob.
fn handle_read_blob(user: &crate::synxit::user::User, req: &super::Request) -> Response {
    match user.read_blob(req.blob_id()) {
        Ok(blob) => Response::success(json!({
            "content": blob.0,
            "hash": blob.1,
        })),
        Err(e) => Response::error(e.to_string().as_str()),
    }
}

/// Handles updating an existing blob.
fn handle_update_blob(user: &crate::synxit::user::User, req: &super::Request) -> Response {
    match user.update_blob(req.blob_id(), req.content(), req.blob_hash()) {
        Ok(new_hash) => Response::success(json!({ "hash": new_hash })),
        Err(e) => Response::error(e.to_string().as_str()),
    }
}

/// Handles deleting an existing blob.
fn handle_delete_blob(user: &crate::synxit::user::User, req: &super::Request) -> Response {
    let success = user.delete_blob(req.blob_id());
    if success {
        Response::success(json!({}))
    } else {
        Response::error("Failed to delete blob")
    }
}

/// Retrieves the hash of a blob.
fn handle_blob_hash(user: &crate::synxit::user::User, req: &super::Request) -> Response {
    match user.read_blob(req.blob_id()) {
        Ok(blob) => Response::success(json!({ "hash": blob.1 })),
        Err(e) => Response::error(e.to_string().as_str()),
    }
}

/// Sets the blob map for the user.
fn handle_set_blob_map(user: &mut crate::synxit::user::User, req: &super::Request) -> Response {
    let blob_map = req.data["blob_map"].as_str();
    match blob_map {
        Some(map) => {
            user.auth.encrypted.blob_map = map.to_string();
            if user.save() {
                Response::success(json!({}))
            } else {
                Response::error("Failed to save blob map")
            }
        }
        None => Response::error("Missing blob map"),
    }
}

/// Retrieves the blob map for the user.
fn handle_get_blob_map(user: &crate::synxit::user::User) -> Response {
    Response::success(json!({
        "blob_map": user.auth.encrypted.blob_map,
    }))
}

/// Retrieves the user's quota information.
fn handle_get_quota(user: &crate::synxit::user::User) -> Response {
    let used = user.get_used_quota();
    let total = user.get_tier_quota();
    Response::success(json!({
        "used": used,
        "total": total,
    }))
}
