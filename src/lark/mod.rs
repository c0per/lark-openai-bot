use axum::{http::StatusCode, routing::post, Json, Router};
use serde_json::{self, json, Map};

fn handle_verification(
    payload: Map<String, serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match payload.get("challenge") {
        Some(c) => Ok(Json(json!({ "challenge": c }))),
        None => Err(StatusCode::BAD_REQUEST),
    }
}

fn handle_event() -> Result<Json<serde_json::Value>, StatusCode> {
    todo!()
}

async fn post_event(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let serde_json::Value::Object(payload) = payload else {
        return Err(StatusCode::BAD_REQUEST);
    };

    if Some((
        &String::from("type"),
        &String::from("url_verification").into(),
    )) == payload.get_key_value("type")
    {
        handle_verification(payload)
    } else {
        handle_event()
    }
}

pub(super) fn router() -> Router {
    Router::new().route("/event", post(post_event))
}
