use super::{AvailabilityResult, RegistryType};
use reqwest::StatusCode;

const FLATHUB_API_URL: &str = "https://flathub.org/api/v1/apps";

/// Check if an app name is available on Flathub (Flatpak)
///
/// API: GET https://flathub.org/api/v1/apps
/// Returns list of all apps; we check if name matches any app
pub async fn check(name: &str) -> AvailabilityResult {
  // Try searching via the apps endpoint with query
  let url = format!("{}/search/{}", FLATHUB_API_URL, name);

  let client = reqwest::Client::new();
  match client
    .get(&url)
    .header("Accept", "application/json")
    .header("User-Agent", "nbi/0.1.0")
    .send()
    .await
  {
    Ok(response) => {
      let status = response.status();

      // If search endpoint doesn't work, try checking if app exists directly
      if status == StatusCode::NOT_FOUND || status == StatusCode::METHOD_NOT_ALLOWED {
        // Try alternative: check apps list
        return check_via_apps_list(name).await;
      }

      if status != StatusCode::OK {
        return AvailabilityResult {
          registry: RegistryType::Flatpak,
          name: name.to_string(),
          available: None,
          error: Some(format!("Status: {}", status)),
        };
      }

      // Parse response to check for matches
      match response.json::<serde_json::Value>().await {
        Ok(json) => {
          let has_match = if let Some(arr) = json.as_array() {
            arr.iter().any(|item| {
              let app_id = item.get("id").or(item.get("flatpakAppId"))
                .and_then(|v| v.as_str()).unwrap_or("");
              let app_name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
              app_id.to_lowercase().contains(&name.to_lowercase())
                || app_name.to_lowercase() == name.to_lowercase()
            })
          } else {
            false
          };

          AvailabilityResult {
            registry: RegistryType::Flatpak,
            name: name.to_string(),
            available: Some(!has_match),
            error: None,
          }
        }
        Err(e) => AvailabilityResult {
          registry: RegistryType::Flatpak,
          name: name.to_string(),
          available: None,
          error: Some(format!("Parse error: {}", e)),
        },
      }
    }
    Err(e) => AvailabilityResult {
      registry: RegistryType::Flatpak,
      name: name.to_string(),
      available: None,
      error: Some(e.to_string()),
    },
  }
}

/// Fallback: fetch apps list and search locally
async fn check_via_apps_list(name: &str) -> AvailabilityResult {
  let url = "https://flathub.org/api/v1/apps";

  let client = reqwest::Client::new();
  match client
    .get(url)
    .header("Accept", "application/json")
    .header("User-Agent", "nbi/0.1.0")
    .send()
    .await
  {
    Ok(response) => {
      if response.status() != StatusCode::OK {
        return AvailabilityResult {
          registry: RegistryType::Flatpak,
          name: name.to_string(),
          available: None,
          error: Some(format!("Status: {}", response.status())),
        };
      }

      match response.json::<Vec<serde_json::Value>>().await {
        Ok(apps) => {
          let name_lower = name.to_lowercase();
          let has_match = apps.iter().any(|app| {
            let app_id = app.get("flatpakAppId")
              .and_then(|v| v.as_str()).unwrap_or("");
            let app_name = app.get("name")
              .and_then(|v| v.as_str()).unwrap_or("");
            app_id.to_lowercase().contains(&name_lower)
              || app_name.to_lowercase() == name_lower
          });

          AvailabilityResult {
            registry: RegistryType::Flatpak,
            name: name.to_string(),
            available: Some(!has_match),
            error: None,
          }
        }
        Err(e) => AvailabilityResult {
          registry: RegistryType::Flatpak,
          name: name.to_string(),
          available: None,
          error: Some(format!("Parse error: {}", e)),
        },
      }
    }
    Err(e) => AvailabilityResult {
      registry: RegistryType::Flatpak,
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
  async fn test_check_existing_app() {
    let result = check("firefox").await;
    // Firefox exists on Flathub
    assert!(result.available == Some(false) || result.error.is_some());
  }

  #[tokio::test]
  async fn test_check_nonexistent_app() {
    let result = check("xyznonexistentapp123456").await;
    assert!(result.available == Some(true) || result.error.is_some());
  }
}
