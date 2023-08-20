#![feature(exit_status_error)]
use axum::http::header::CACHE_CONTROL;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::{http::StatusCode, routing::post, Json, Router};
use owner_tree::build_owner_tree;
use parse::emacs_parse_org_document;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

mod error;
mod owner_tree;
mod parse;
mod rtrim_iterator;
mod sexp;

#[tokio::main]
async fn main() {
    let static_files_service = {
        let serve_dir =
            ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));

        ServiceBuilder::new()
            .layer(SetResponseHeaderLayer::if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=120"),
            ))
            .service(serve_dir)
    };
    let app = Router::new()
        .route("/parse", post(parse_org_mode))
        .fallback_service(static_files_service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000. Pop open your browser to http://127.0.0.1:3000/ .");
    axum::serve(listener, app).await.unwrap();
}

async fn parse_org_mode(body: String) -> Result<impl IntoResponse, (StatusCode, String)> {
    _parse_org_mode(body)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn _parse_org_mode(body: String) -> Result<impl IntoResponse, Box<dyn std::error::Error>> {
    let ast = emacs_parse_org_document(&body).await?;
    let owner_tree = build_owner_tree(body.as_str(), ast.as_str()).map_err(|e| e.to_string())?;
    Ok((StatusCode::OK, Json(owner_tree)))
}
