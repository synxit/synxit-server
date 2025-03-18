use super::{parse_request, Response};
pub fn handle_blob(body: String) -> Response {
    let req = parse_request(body);
    match req.get_auth_user() {
        Ok(mut user) => match req.action.as_str() {
            "new_blob" => {
                let blob = user.new_blob(req.data["content"].as_str().unwrap());
                Response {
                    success: blob.success,
                    data: serde_json::json!({
                        "id": blob.id,
                        "hash": blob.hash,
                    }),
                }
            }
            "read_blob" => {
                let blob = user.read_blob(req.data["id"].as_str().unwrap());
                Response {
                    success: blob.success,
                    data: serde_json::json!({
                        "content": blob.content,
                        "hash": blob.hash,
                    }),
                }
            }
            "update_blob" => {
                let blob = user.update_blob(
                    req.data["id"].as_str().unwrap(),
                    req.data["content"].as_str().unwrap(),
                    req.data["hash"].as_str().unwrap(),
                );
                Response {
                    success: blob.success,
                    data: serde_json::json!({
                        "hash": blob.hash,
                    }),
                }
            }
            "delete_blob" => {
                user.delete_blob(req.data["id"].as_str().unwrap());
                Response {
                    success: true,
                    data: serde_json::json!({
                        "message": "Deleted"
                    }),
                }
            }
            "set_blob_map" => {
                user.auth.encrypted.blob_map =
                    serde_json::to_string(&req.data["blob_map"]).unwrap();
                user.save();
                Response {
                    success: true,
                    data: serde_json::json!({
                        "message": "Set"
                    }),
                }
            }
            _ => Response {
                success: false,
                data: serde_json::json!({
                    "error": "Invalid action"
                }),
            },
        },
        Err(err) => err,
    }
}
