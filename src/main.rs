use axum::Router;

mod lark;

#[tokio::main]
async fn main() {
    let app = Router::new().nest("/lark", lark::router());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
