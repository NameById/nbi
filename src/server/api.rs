use axum::{
  http::StatusCode,
  response::{Html, IntoResponse},
  Json,
};
use serde::{Deserialize, Serialize};

use crate::config::{Config, RegistrySettings};
use crate::registry::{self, AvailabilityResult};

/// Index page with embedded React app
pub async fn index() -> Html<&'static str> {
  Html(include_str!("../../static/index.html"))
}

#[derive(Deserialize)]
pub struct CheckRequest {
  pub name: String,
  #[serde(default)]
  pub registries: Option<RegistrySettings>,
}

#[derive(Serialize)]
pub struct CheckResponse {
  pub name: String,
  pub results: Vec<AvailabilityResult>,
}

/// Check package name availability
pub async fn check_availability(Json(req): Json<CheckRequest>) -> impl IntoResponse {
  let settings = req.registries.unwrap_or_default();
  let results = registry::check_all(&req.name, &settings).await;

  Json(CheckResponse {
    name: req.name,
    results,
  })
}

#[derive(Deserialize)]
pub struct DomainRequest {
  pub name: String,
  pub tlds: Vec<String>,
}

#[derive(Serialize)]
pub struct DomainResponse {
  pub name: String,
  pub results: Vec<DomainResult>,
}

#[derive(Serialize)]
pub struct DomainResult {
  pub domain: String,
  pub available: Option<bool>,
  pub error: Option<String>,
}

/// Check domain availability across multiple TLDs
pub async fn check_domain(Json(req): Json<DomainRequest>) -> impl IntoResponse {
  use crate::registry::domain::check_tld;

  let mut results = Vec::new();

  for tld in &req.tlds {
    let domain = format!("{}.{}", req.name, tld);
    let result = check_tld(&req.name, tld).await;
    results.push(DomainResult {
      domain,
      available: result.available,
      error: result.error,
    });
  }

  Json(DomainResponse {
    name: req.name,
    results,
  })
}

#[derive(Deserialize)]
pub struct FullDomainRequest {
  pub domains: Vec<String>,
}

/// Check full domain availability (e.g., banana.wiki)
pub async fn check_full_domains(Json(req): Json<FullDomainRequest>) -> impl IntoResponse {
  use crate::registry::domain::check_full_domain;

  let mut results = Vec::new();

  for domain in &req.domains {
    let result = check_full_domain(domain).await;
    results.push(DomainResult {
      domain: domain.clone(),
      available: result.available,
      error: result.error,
    });
  }

  Json(DomainResponse {
    name: req.domains.join(", "),
    results,
  })
}

/// Get current config
pub async fn get_config() -> impl IntoResponse {
  match Config::load() {
    Ok(config) => (StatusCode::OK, Json(serde_json::to_value(config).unwrap())),
    Err(e) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({ "error": e.to_string() })),
    ),
  }
}

#[derive(Deserialize)]
pub struct SaveConfigRequest {
  pub registries: RegistrySettings,
}

/// Save config
pub async fn save_config(Json(req): Json<SaveConfigRequest>) -> impl IntoResponse {
  let mut config = Config::load().unwrap_or_default();
  config.registries = req.registries;

  match config.save() {
    Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))),
    Err(e) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({ "error": e.to_string() })),
    ),
  }
}
