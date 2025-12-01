pub mod brew;
pub mod crates;
pub mod debian;
pub mod domain;
pub mod flatpak;
pub mod github;
pub mod npm;
pub mod pypi;

use serde::{Deserialize, Serialize};

/// Availability check result for a registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityResult {
  pub registry: RegistryType,
  pub name: String,
  pub available: Option<bool>, // None = check failed
  pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryType {
  Npm,
  Crates,
  PyPi,
  Brew,
  Flatpak,
  Debian,
  DevDomain,
  GitHub,
}

impl std::fmt::Display for RegistryType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RegistryType::Npm => write!(f, "npm"),
      RegistryType::Crates => write!(f, "crates.io"),
      RegistryType::PyPi => write!(f, "PyPI"),
      RegistryType::Brew => write!(f, "Homebrew"),
      RegistryType::Flatpak => write!(f, "Flatpak"),
      RegistryType::Debian => write!(f, "Debian"),
      RegistryType::DevDomain => write!(f, ".dev"),
      RegistryType::GitHub => write!(f, "GitHub"),
    }
  }
}

use crate::config::RegistrySettings;

/// Check availability across enabled registries
pub async fn check_all(name: &str, settings: &RegistrySettings) -> Vec<AvailabilityResult> {
  let mut results = Vec::new();

  let (npm_res, crates_res, pypi_res, brew_res, flatpak_res, debian_res, domain_res) = tokio::join!(
    async { if settings.npm { Some(npm::check(name).await) } else { None } },
    async { if settings.crates { Some(crates::check(name).await) } else { None } },
    async { if settings.pypi { Some(pypi::check(name).await) } else { None } },
    async { if settings.brew { Some(brew::check(name).await) } else { None } },
    async { if settings.flatpak { Some(flatpak::check(name).await) } else { None } },
    async { if settings.debian { Some(debian::check(name).await) } else { None } },
    async { if settings.dev_domain { Some(domain::check(name).await) } else { None } },
  );

  if let Some(r) = npm_res { results.push(r); }
  if let Some(r) = crates_res { results.push(r); }
  if let Some(r) = pypi_res { results.push(r); }
  if let Some(r) = brew_res { results.push(r); }
  if let Some(r) = flatpak_res { results.push(r); }
  if let Some(r) = debian_res { results.push(r); }
  if let Some(r) = domain_res { results.push(r); }

  results
}
