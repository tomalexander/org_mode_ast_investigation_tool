#![feature(exit_status_error)]
use axum::{http::StatusCode, routing::post, Json, Router};
use parse::emacs_parse_org_document;
use serde::Serialize;
use tower_http::services::{ServeDir, ServeFile};

mod owner_tree;
mod parse;

#[tokio::main]
async fn main() {
    let serve_dir = ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));
    let app = Router::new()
        .route("/parse", post(parse_org_mode))
        .fallback_service(serve_dir);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn parse_org_mode(body: String) -> Result<(StatusCode, Json<OwnerTree>), (StatusCode, String)> {
    let ast = emacs_parse_org_document(&body).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let ret = OwnerTree { input_source: body };
    Ok((StatusCode::OK, Json(ret)))
}

#[derive(Serialize)]
struct OwnerTree {
    input_source: String,
}
