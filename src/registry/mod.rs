pub mod crates;
pub mod domain;
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
  DevDomain,
  GitHub,
}

impl std::fmt::Display for RegistryType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RegistryType::Npm => write!(f, "npm"),
      RegistryType::Crates => write!(f, "crates.io"),
      RegistryType::PyPi => write!(f, "PyPI"),
      RegistryType::DevDomain => write!(f, ".dev"),
      RegistryType::GitHub => write!(f, "GitHub"),
    }
  }
}

/// Check availability across all registries
pub async fn check_all(name: &str) -> Vec<AvailabilityResult> {
  let (npm, crates, pypi, domain) = tokio::join!(
    npm::check(name),
    crates::check(name),
    pypi::check(name),
    domain::check(name),
  );
  vec![npm, crates, pypi, domain]
}
