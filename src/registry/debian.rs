use super::{AvailabilityResult, RegistryType};
use reqwest::StatusCode;

const DEBIAN_API_URL: &str = "https://sources.debian.org/api/src";

/// Check if a package name is available on Debian
///
/// API: GET https://sources.debian.org/api/src/{name}/
/// - 200 with versions: Package exists (not available)
/// - 200 with error: Package not found (available)
/// - 404: Package not found (available)
pub async fn check(name: &str) -> AvailabilityResult {
  let url = format!("{}/{}/", DEBIAN_API_URL, name);

  match reqwest::get(&url).await {
    Ok(response) => {
      let status = response.status();

      if status == StatusCode::NOT_FOUND {
        return AvailabilityResult {
          registry: RegistryType::Debian,
          name: name.to_string(),
          available: Some(true),
          error: None,
        };
      }

      if status != StatusCode::OK {
        return AvailabilityResult {
          registry: RegistryType::Debian,
          name: name.to_string(),
          available: None,
          error: Some(format!("Unexpected status: {}", status)),
        };
      }

      // Parse response - check if package has versions
      match response.json::<serde_json::Value>().await {
        Ok(json) => {
          // If there's an error field, package doesn't exist
          if json.get("error").is_some() {
            return AvailabilityResult {
              registry: RegistryType::Debian,
              name: name.to_string(),
              available: Some(true),
              error: None,
            };
          }

          // Check for versions array
          let has_versions = json
            .get("versions")
            .and_then(|v| v.as_array())
            .map_or(false, |arr| !arr.is_empty());

          AvailabilityResult {
            registry: RegistryType::Debian,
            name: name.to_string(),
            available: Some(!has_versions),
            error: None,
          }
        }
        Err(e) => AvailabilityResult {
          registry: RegistryType::Debian,
          name: name.to_string(),
          available: None,
          error: Some(format!("Parse error: {}", e)),
        },
      }
    }
    Err(e) => AvailabilityResult {
      registry: RegistryType::Debian,
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
    let result = check("bash").await;
    assert_eq!(result.available, Some(false));
  }

  #[tokio::test]
  async fn test_check_nonexistent_package() {
    let result = check("this-package-definitely-does-not-exist-xyz123abc").await;
    assert_eq!(result.available, Some(true));
  }
}
