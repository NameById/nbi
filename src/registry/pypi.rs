use super::{AvailabilityResult, RegistryType};
use reqwest::StatusCode;

const PYPI_SIMPLE_URL: &str = "https://pypi.org/simple";

/// Check if a package name is available on PyPI
///
/// API: GET https://pypi.org/simple/{name}/
/// - 200: Package exists (not available)
/// - 404: Package not found (available)
///
/// Note: Using /simple/ endpoint as it correctly returns 404 for
/// packages that are registered but have no releases
pub async fn check(name: &str) -> AvailabilityResult {
  let url = format!("{}/{}/", PYPI_SIMPLE_URL, name);

  match reqwest::get(&url).await {
    Ok(response) => {
      let available = match response.status() {
        StatusCode::NOT_FOUND => Some(true),
        StatusCode::OK => Some(false),
        _ => None,
      };
      AvailabilityResult {
        registry: RegistryType::PyPi,
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
      registry: RegistryType::PyPi,
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
  async fn test_check_existing_package() {
    let result = check("requests").await;
    assert_eq!(result.available, Some(false));
  }

  #[tokio::test]
  async fn test_check_nonexistent_package() {
    let result = check("this-package-definitely-does-not-exist-xyz123abc").await;
    assert_eq!(result.available, Some(true));
  }
}
