use axum::{http::StatusCode, routing::post, Json, Router};
use serde::Serialize;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let serve_dir = ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));
    let app = Router::new()
        .route("/parse", post(parse_org_mode))
        .fallback_service(serve_dir);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn parse_org_mode(body: String) -> (StatusCode, Json<OwnerTree>) {
    let ret = OwnerTree { input_source: body };
    (StatusCode::OK, Json(ret))
}

#[derive(Serialize)]
struct OwnerTree {
    input_source: String,
}
