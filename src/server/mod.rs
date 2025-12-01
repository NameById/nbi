mod api;

use anyhow::Result;
use axum::{
  routing::{get, post},
  Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

pub async fn start(port: u16, open_browser: bool) -> Result<()> {
  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

  let app = Router::new()
    .route("/", get(api::index))
    .route("/api/check", post(api::check_availability))
    .route("/api/domain", post(api::check_domain))
    .route("/api/domain/full", post(api::check_full_domains))
    .route("/api/config", get(api::get_config))
    .route("/api/config", post(api::save_config))
    .layer(cors);

  let addr = SocketAddr::from(([127, 0, 0, 1], port));
  println!("🚀 Server running at http://{}", addr);

  if open_browser {
    let url = format!("http://{}", addr);
    if let Err(e) = open::that(&url) {
      eprintln!("Failed to open browser: {}", e);
    }
  }

  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}
