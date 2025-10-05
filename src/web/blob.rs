use crate::logger::error::ERROR_INVALID_ACTION;

use super::{parse_request, Response};
use serde_json::json;
pub fn handle_blob(body: String) -> Response {
    let req = parse_request(body);
    match req.get_auth_user() {
        Ok(mut user) => match req.action.as_str() {
            "new" => match user.new_blob(req.data["content"].as_str().unwrap_or_default()) {
                Ok(blob) => Response::success(json!({
                    "id": blob.0,
                    "hash": blob.1,
                })),
                Err(e) => Response::error(e.to_string().as_str()),
            },
            "get" => match user.read_blob(req.data["id"].as_str().unwrap_or_default()) {
                Ok(blob) => Response::success(json!({
                  "content": blob.0,
                  "hash": blob.1,
                })),
                Err(e) => Response::error(e.to_string().as_str()),
            },
            "update" => {
                match user.update_blob(
                    req.data["id"].as_str().unwrap_or_default(),
                    req.data["content"].as_str().unwrap_or_default(),
                    req.data["hash"].as_str().unwrap_or_default(),
                ) {
                    Ok(blob) => Response::success(json!({
                        "hash": blob
                    })),
                    Err(e) => Response::error(e.to_string().as_str()),
                }
            }
            "delete" => Response {
                success: user.delete_blob(req.data["id"].as_str().unwrap_or_default()),
                data: json!({}),
            },
            "set_blob_map" => match req.data["blob_map"].as_str() {
                Some(blob_map) => {
                    user.auth.encrypted.blob_map = blob_map.to_string();
                    if user.save() {
                        Response::success(json!({}))
                    } else {
                        Response::error("SAVE_ERROR")
                    }
                }
                None => Response::error("MISSING_BLOB_MAP"),
            },
            "get_quota" => {
                let used = user.get_used_quota();
                let total = user.get_tier_quota();
                Response::success(json!({
                    "used": used,
                    "total": total,
                }))
            }
            _ => Response::error(ERROR_INVALID_ACTION),
        },
        Err(err) => err,
    }
}
