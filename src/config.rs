use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "nbi";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
  pub github_token: Option<String>,
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
  #[allow(dead_code)]
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

  /// Set GitHub token and save
  #[allow(dead_code)]
  pub fn set_github_token(&mut self, token: String) -> Result<()> {
    self.github_token = Some(token);
    self.save()
  }

  /// Get GitHub token from config or environment
  pub fn get_github_token(&self) -> Option<String> {
    self.github_token
      .clone()
      .or_else(|| std::env::var("GITHUB_TOKEN").ok())
  }
}
