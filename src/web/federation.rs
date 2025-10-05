use serde_json::json;

use super::{parse_request, Response};
use crate::{
    logger::error::{
        ERROR_BLOB_HASH_NOT_MATCH, ERROR_BLOB_NOT_FOUND, ERROR_BLOB_NOT_IN_SHARE,
        ERROR_INVALID_ACTION, ERROR_INVALID_JSON, ERROR_NO_WRITE_ACCESS, ERROR_REMOTE_ERROR,
        ERROR_USER_NOT_FOUND, ERROR_WRONG_SECRET,
    },
    synxit::{
        config::{Config, CONFIG},
        user::User,
    },
    utils::char_hex_string_to_u128,
};

pub async fn handle_federation(body: String) -> Response {
    let req = parse_request(body.to_string());
    let share_user = req.data["share_user"].as_str().unwrap_or_default();
    let config_default = Config::default();
    let config = CONFIG.get().unwrap_or(&config_default);
    let server = User::resolve_user(share_user)
        .unwrap_or((String::new(), String::new()))
        .1;
    if config.federation.enabled {
        if config.federation.whitelist.enabled {
            if !config.federation.whitelist.hosts.contains(&server) {
                return Response::error(ERROR_REMOTE_ERROR);
            }
        }
        if config.federation.blacklist.enabled {
            if config.federation.blacklist.hosts.contains(&server) {
                return Response::error(ERROR_REMOTE_ERROR);
            }
        }
    } else {
        return Response::error(ERROR_REMOTE_ERROR);
    }
    let share_secret = req.data["secret"].as_str().unwrap_or_default();
    let share_secret_num = char_hex_string_to_u128(share_secret.to_string());
    let share_id = req.data["id"].as_str().unwrap_or_default();
    match req.action.as_str() {
        "proxy" => {
            handle_proxy_action(parse_request(body), &share_user, share_secret, share_id).await
        }
        "blobs" => handle_blobs_action(&share_user, share_secret_num, share_id),
        "get" => handle_get_action(&req, &share_user, share_secret_num, share_id),
        "update" => handle_update_action(&req, &share_user, share_secret_num, share_id),
        "delete" => handle_delete_action(&req, &share_user, share_secret_num, share_id),
        "new" => handle_new_action(&req, &share_user, share_secret_num, share_id),
        _ => Response::error(ERROR_INVALID_ACTION),
    }
}

async fn handle_proxy_action(
    req: super::Request,
    share_user: &str,
    share_secret: &str,
    share_id: &str,
) -> Response {
    match req.get_auth_user() {
        Ok(user) => user,
        Err(err) => return err,
    };

    let pair = match User::resolve_user(share_user) {
        Ok(pair) => pair,
        Err(_) => return Response::error(ERROR_INVALID_ACTION),
    };

    let url = format!("http://{}:8400/synxit/federation", pair.1);

    match req.data["action"].as_str().unwrap_or_default() {
        "blobs" => {
            let request_body = json!({
                "action": "blobs",
                "data": {
                    "id": share_id,
                    "share_user": share_user,
                    "secret": share_secret
                }
            });

            match post_request(url, &request_body).await {
                Ok(success) => serde_json::from_value(success)
                    .unwrap_or_else(|_| Response::error(ERROR_INVALID_JSON)),
                Err(_) => Response::error(ERROR_REMOTE_ERROR),
            }
        }
        "get" => {
            let request_body = json!({
                "action": "get",
                "data": {
                    "id": share_id,
                    "share_user": share_user,
                    "secret": share_secret,
                    "blob": req.data["blob"].as_str().unwrap_or_default()
                }
            });

            match post_request(url, &request_body).await {
                Ok(success) => serde_json::from_value(success)
                    .unwrap_or_else(|_| Response::error(ERROR_INVALID_JSON)),
                Err(_) => Response::error(ERROR_REMOTE_ERROR),
            }
        }
        "update" => {
            let request_body = json!({
                "action": "update",
                "data": {
                    "id": share_id,
                    "share_user": share_user,
                    "secret": share_secret,
                    "blob": req.data["blob"].as_str().unwrap_or_default(),
                    "content": req.data["content"].as_str().unwrap_or_default(),
                    "hash": req.data["hash"].as_str().unwrap_or_default(),
                }
            });

            match post_request(url, &request_body).await {
                Ok(success) => serde_json::from_value(success)
                    .unwrap_or_else(|_| Response::error(ERROR_INVALID_JSON)),
                Err(_) => Response::error(ERROR_REMOTE_ERROR),
            }
        }
        "delete" => {
            let request_body = json!({
                "action": "delete",
                "data": {
                    "id": share_id,
                    "share_user": share_user,
                    "secret": share_secret,
                    "blob": req.data["blob"].as_str().unwrap_or_default(),
                }
            });

            match post_request(url, &request_body).await {
                Ok(success) => serde_json::from_value(success)
                    .unwrap_or_else(|_| Response::error(ERROR_INVALID_JSON)),
                Err(_) => Response::error(ERROR_REMOTE_ERROR),
            }
        }
        "new" => {
            let request_body = json!({
                "action": "new",
                "data": {
                    "id": share_id,
                    "share_user": share_user,
                    "secret": share_secret,
                    "content": req.data["content"].as_str().unwrap_or_default(),
                }
            });

            match post_request(url, &request_body).await {
                Ok(success) => serde_json::from_value(success)
                    .unwrap_or_else(|_| Response::error(ERROR_INVALID_JSON)),
                Err(_) => Response::error(ERROR_REMOTE_ERROR),
            }
        }
        _ => Response::error(ERROR_INVALID_ACTION),
    }
}

fn handle_blobs_action(share_user: &str, share_secret_num: u128, share_id: &str) -> Response {
    let share = match User::get_share_by_id(share_user.to_string(), share_id.to_string()) {
        Ok(share) => share,
        Err(err) => return Response::error(&err.to_string()),
    };

    if share.secret != share_secret_num {
        return Response::error(ERROR_WRONG_SECRET);
    }

    Response {
        success: true,
        data: json!({
            "blobs": share.blobs,
            "write_access": share.write
        }),
    }
}

fn handle_get_action(
    req: &super::Request,
    share_user: &str,
    share_secret_num: u128,
    share_id: &str,
) -> Response {
    let blob = req.data["blob"].as_str().unwrap_or_default();

    match validate_share_access(share_user, share_secret_num, share_id) {
        Ok(share) => share,
        Err(response) => return response,
    };

    let user = match User::load(share_user) {
        Ok(user) => user,
        Err(_) => return Response::error(ERROR_USER_NOT_FOUND),
    };

    if let Err(_) = User::check_share_permissions(
        share_user.to_string(),
        share_id.to_string(),
        share_secret_num.to_string(),
        blob.to_string(),
        false,
    ) {
        return Response::error(ERROR_BLOB_NOT_IN_SHARE);
    }

    match user.read_blob(blob) {
        Ok(result) => Response::success(json!({
            "content": result.0,
            "hash": result.1,
        })),
        Err(e) => Response::error(e.to_string().as_str()),
    }
}

fn handle_update_action(
    req: &super::Request,
    share_user: &str,
    share_secret_num: u128,
    share_id: &str,
) -> Response {
    let blob = req.data["blob"].as_str().unwrap_or_default();
    let content = req.data["content"].as_str().unwrap_or_default();
    let old_hash = req.data["hash"].as_str().unwrap_or_default();

    match validate_share_access(share_user, share_secret_num, share_id) {
        Ok(share) => share,
        Err(response) => return response,
    };

    let user = match User::load(share_user) {
        Ok(user) => user,
        Err(_) => return Response::error(ERROR_USER_NOT_FOUND),
    };

    if let Err(_) = User::check_share_permissions(
        share_user.to_string(),
        share_id.to_string(),
        share_secret_num.to_string(),
        blob.to_string(),
        true,
    ) {
        return Response::error(ERROR_BLOB_NOT_IN_SHARE);
    }

    match user.update_blob(blob, content, old_hash) {
        Ok(hash) => Response::success(json!({
            "hash": hash
        })),
        Err(e) => Response::error(e.to_string().as_str()),
    }
}

fn handle_delete_action(
    req: &super::Request,
    share_user: &str,
    share_secret_num: u128,
    share_id: &str,
) -> Response {
    let blob = req.data["blob"].as_str().unwrap_or_default();
    let hash = req.data["hash"].as_str().unwrap_or_default();

    match validate_share_access(share_user, share_secret_num, share_id) {
        Ok(share) => share,
        Err(response) => return response,
    };

    let user = match User::load(share_user) {
        Ok(user) => user,
        Err(_) => return Response::error(ERROR_USER_NOT_FOUND),
    };

    if let Err(_) = User::check_share_permissions(
        share_user.to_string(),
        share_id.to_string(),
        share_secret_num.to_string(),
        blob.to_string(),
        true,
    ) {
        return Response::error(ERROR_BLOB_NOT_IN_SHARE);
    }

    match user.read_blob(blob) {
        Ok(read) => {
            if hash.to_string() == read.1 {
                if user.delete_blob(blob) {
                    Response::success(json!({}))
                } else {
                    Response::error(ERROR_BLOB_NOT_FOUND)
                }
            } else {
                Response::error(ERROR_BLOB_HASH_NOT_MATCH)
            }
        }
        Err(e) => Response::error(e.to_string().as_str()),
    }
}

fn handle_new_action(
    req: &super::Request,
    share_user: &str,
    share_secret_num: u128,
    share_id: &str,
) -> Response {
    let content = req.data["content"].as_str().unwrap_or_default();

    let share = match validate_share_access(share_user, share_secret_num, share_id) {
        Ok(share) => share,
        Err(response) => return response,
    };

    if !share.write {
        return Response::error(ERROR_NO_WRITE_ACCESS);
    }

    let user = match User::load(share_user) {
        Ok(user) => user,
        Err(_) => return Response::error(ERROR_USER_NOT_FOUND),
    };

    let new_blob = match user.new_blob(content) {
        Ok(blob) => blob,
        Err(err) => return Response::error(&err.to_string()),
    };

    if let Err(_) = User::add_blob_to_share(
        share_user.to_string(),
        share_id.to_string(),
        new_blob.0.to_string(),
    ) {
        Response::error(ERROR_BLOB_NOT_IN_SHARE)
    } else {
        Response::success(json!({
            "id": new_blob.0
        }))
    }
}

fn validate_share_access(
    share_user: &str,
    share_secret_num: u128,
    share_id: &str,
) -> Result<crate::synxit::user::blob::Share, Response> {
    let share = match User::get_share_by_id(share_user.to_string(), share_id.to_string()) {
        Ok(share) => share,
        Err(err) => return Err(Response::error(&err.to_string())),
    };

    if share.secret != share_secret_num {
        return Err(Response::error(ERROR_WRONG_SECRET));
    }

    Ok(share)
}

async fn post_request(
    url: String,
    json_body: &serde_json::Value,
) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(url)
        .json(json_body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await
}
