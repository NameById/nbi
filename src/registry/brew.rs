use super::{AvailabilityResult, RegistryType};
use reqwest::StatusCode;

const BREW_API_URL: &str = "https://formulae.brew.sh/api/formula";

/// Check if a formula name is available on Homebrew
///
/// API: GET https://formulae.brew.sh/api/formula/{name}.json
/// - 200: Formula exists (not available)
/// - 404: Formula not found (available)
pub async fn check(name: &str) -> AvailabilityResult {
  let url = format!("{}/{}.json", BREW_API_URL, name);

  match reqwest::get(&url).await {
    Ok(response) => {
      let available = match response.status() {
        StatusCode::NOT_FOUND => Some(true),
        StatusCode::OK => Some(false),
        _ => None,
      };
      AvailabilityResult {
        registry: RegistryType::Brew,
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
      registry: RegistryType::Brew,
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
  async fn test_check_existing_formula() {
    let result = check("git").await;
    assert_eq!(result.available, Some(false));
  }

  #[tokio::test]
  async fn test_check_nonexistent_formula() {
    let result = check("this-formula-definitely-does-not-exist-xyz123abc").await;
    assert_eq!(result.available, Some(true));
  }
}
