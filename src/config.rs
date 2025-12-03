use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "nbi";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySettings {
  #[serde(default = "default_true")]
  pub npm: bool,
  #[serde(default = "default_true")]
  pub crates: bool,
  #[serde(default = "default_true")]
  pub pypi: bool,
  #[serde(default = "default_true")]
  pub brew: bool,
  #[serde(default = "default_true")]
  pub flatpak: bool,
  #[serde(default = "default_true")]
  pub debian: bool,
  #[serde(default = "default_true")]
  pub dev_domain: bool,
  #[serde(default = "default_true")]
  pub github: bool,
}

fn default_true() -> bool {
  true
}

impl Default for RegistrySettings {
  fn default() -> Self {
    Self {
      npm: true,
      crates: true,
      pypi: true,
      brew: true,
      flatpak: true,
      debian: true,
      dev_domain: true,
      github: true,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
  #[serde(skip)]
  #[allow(dead_code)]
  github_token: Option<String>,
  #[serde(default)]
  pub registries: RegistrySettings,
}

impl Config {
  /// Get the config file path
  fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", APP_NAME).map(|dirs| dirs.config_dir().join("config.toml"))
  }

  /// Load config from file
  pub fn load() -> Result<Self> {
    let path =
      Self::config_path().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

    if !path.exists() {
      return Ok(Self::default());
    }

    let content = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
  }

  /// Save config to file
  pub fn save(&self) -> Result<()> {
    let path =
      Self::config_path().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(self)?;
    fs::write(&path, content)?;
    Ok(())
  }

  /// GitHub token is no longer stored in config file for security
  #[allow(dead_code)]
  pub fn set_github_token(&mut self, _token: String) -> Result<()> {
    // Deprecated: tokens should only be provided via environment variables
    anyhow::bail!("GitHub tokens should only be set via GITHUB_TOKEN environment variable for security")
  }

  /// Get GitHub token from environment only (not stored in config)
  pub fn get_github_token(&self) -> Option<String> {
    std::env::var("GITHUB_TOKEN").ok()
  }
}
