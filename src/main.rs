#![feature(exit_status_error)]
use axum::{http::StatusCode, routing::post, Json, Router};
use owner_tree::{build_owner_tree, OwnerTree};
use parse::emacs_parse_org_document;
use tower_http::services::{ServeDir, ServeFile};

mod error;
mod owner_tree;
mod parse;
mod sexp;

#[tokio::main]
async fn main() {
    let serve_dir = ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));
    let app = Router::new()
        .route("/parse", post(parse_org_mode))
        .fallback_service(serve_dir);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn parse_org_mode(
    body: String,
) -> Result<(StatusCode, Json<OwnerTree>), (StatusCode, String)> {
    _parse_org_mode(body)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn _parse_org_mode(
    body: String,
) -> Result<(StatusCode, Json<OwnerTree>), Box<dyn std::error::Error>> {
    let ast = emacs_parse_org_document(&body).await?;
    let owner_tree = build_owner_tree(body.as_str(), ast.as_str()).map_err(|e| e.to_string())?;
    Ok((StatusCode::OK, Json(owner_tree)))
}
