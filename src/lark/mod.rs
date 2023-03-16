use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde_json::{self, json, Map};

use crate::AppState;

async fn handle_message(state: AppState, event: &Map<String, serde_json::Value>) {
    let open_id = event
        .get("sender")
        .unwrap()
        .get("sender_id")
        .unwrap()
        .get("open_id")
        .unwrap()
        .as_str()
        .unwrap();

    let content = event
        .get("message")
        .unwrap()
        .get("content")
        .unwrap()
        .as_str()
        .unwrap();

    let client = reqwest::Client::new();
    client
        .post("https://open.feishu.cn/open-apis/im/v1/messages")
        .bearer_auth(state.lock().await.tenant_token.get_token().await)
        .query(&[("receive_id_type", "open_id")])
        .json(&json!({
            "receive_id": open_id,
            "msg_type": "text",
            "content": content
        }))
        .send()
        .await
        .unwrap();
}

fn handle_verification(
    payload: Map<String, serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match payload.get("challenge") {
        Some(c) => Ok(Json(json!({ "challenge": c }))),
        None => Err(StatusCode::BAD_REQUEST),
    }
}

async fn handle_event_v2(
    state: AppState,
    payload: Map<String, serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let header = payload.get("header").unwrap().as_object().unwrap();
    let event = payload.get("event").unwrap().as_object().unwrap();

    let event_type = header.get("event_type").unwrap().as_str().unwrap();
    match event_type {
        "im.message.receive_v1" => handle_message(state, event).await,
        _ => unimplemented!(),
    }

    Ok(Json(serde_json::Value::Null))
}

fn handle_event_v1() -> Result<Json<serde_json::Value>, StatusCode> {
    todo!()
}

async fn post_event(
    State(state): State<AppState>,
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
        match payload.get("schema") {
            Some(_) => handle_event_v2(state, payload).await,
            None => handle_event_v1(),
        }
    }
}

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/event", post(post_event))
}
