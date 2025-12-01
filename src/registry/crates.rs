use super::{AvailabilityResult, RegistryType};
use reqwest::StatusCode;

const CRATES_API_URL: &str = "https://crates.io/api/v1/crates";

/// Check if a crate name is available on crates.io
///
/// API: GET https://crates.io/api/v1/crates/{name}
/// - 200: Crate exists (not available)
/// - 404: Crate not found (available)
///
/// Note: crates.io requires a User-Agent header
pub async fn check(name: &str) -> AvailabilityResult {
  let url = format!("{}/{}", CRATES_API_URL, name);

  let client = reqwest::Client::new();
  match client
    .get(&url)
    .header("User-Agent", "nbi/0.1.0 (package-name-checker)")
    .send()
    .await
  {
    Ok(response) => {
      let available = match response.status() {
        StatusCode::NOT_FOUND => Some(true),
        StatusCode::OK => Some(false),
        _ => None,
      };
      AvailabilityResult {
        registry: RegistryType::Crates,
        name: name.to_string(),
        available,
        error: if available.is_none() {
          Some(format!("Unexpected status: {}", response.status()))
        } else {
          None
        },
      }
    }
    Err(e) => AvailabilityResult {
      registry: RegistryType::Crates,
      name: name.to_string(),
      available: None,
      error: Some(e.to_string()),
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_check_existing_crate() {
    let result = check("serde").await;
    assert_eq!(result.available, Some(false));
  }

  #[tokio::test]
  async fn test_check_nonexistent_crate() {
    let result = check("this-crate-definitely-does-not-exist-xyz123abc").await;
    assert_eq!(result.available, Some(true));
  }
}
