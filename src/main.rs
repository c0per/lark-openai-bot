use axum::Router;
use serde_json::json;
use std::{
    env,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

mod lark;

#[derive(Clone)]
struct TenantToken {
    token: String,
    expire_timestamp: u64,
}

impl TenantToken {
    async fn init() -> TenantToken {
        get_tenant_token().await
    }

    async fn get_token(&mut self) -> &str {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > self.expire_timestamp - 30 * 60 {
            *self = get_tenant_token().await;
        }
        &self.token
    }
}

#[derive(Clone)]
struct AppStateInner {
    tenant_token: TenantToken,
    openai: async_openai::Client,
}

type AppState = Arc<Mutex<AppStateInner>>;

async fn get_tenant_token() -> TenantToken {
    let client = reqwest::Client::new();
    let res = client
        .post("https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal")
        .json(&json!({
            "app_id": env::var("LARK_APP_ID").unwrap(),
            "app_secret": env::var("LARK_APP_SECRET").unwrap()
        }))
        .send()
        .await
        .unwrap();

    let json = res.json::<serde_json::Value>().await.unwrap();
    let json = json.as_object().unwrap();

    let token = json
        .get("tenant_access_token")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let expiration = json.get("expire").unwrap().as_u64().unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    TenantToken {
        token,
        expire_timestamp: now + expiration,
    }
}

#[tokio::main]
async fn main() {
    let openai = async_openai::Client::new();

    let app = Router::new().nest(
        "/lark",
        lark::router().with_state(Arc::new(Mutex::new(AppStateInner {
            tenant_token: TenantToken::init().await,
            openai,
        }))),
    );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
