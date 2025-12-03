//! TUI input handlers with clean architecture
//! 
//! This module handles user input for different screens in the TUI application.
//! Each handler is responsible for a specific screen and delegates business logic
//! to appropriate services.

use crate::app::{App, InputMode};
use crate::registry::{self, RegistryType, github::{ManifestType, GitHubError}};
use crossterm::event::KeyCode;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Result type for registration operations
#[derive(Debug, Clone)]
pub enum RegistrationResult {
  Success(String),
  Error(String),
}

/// Handle search screen input
pub async fn handle_search_input(
  app: &mut App,
  key_code: KeyCode,
  app_arc: Arc<Mutex<App>>,
) {
  match app.input_mode {
    InputMode::Normal => handle_search_normal_mode(app, key_code),
    InputMode::Editing => handle_search_editing_mode(app, key_code, app_arc).await,
  }
}

fn handle_search_normal_mode(app: &mut App, key_code: KeyCode) {
  match key_code {
    KeyCode::Char('i') | KeyCode::Char('e') | KeyCode::Enter => {
      app.input_mode = InputMode::Editing;
    }
    KeyCode::Up => app.select_previous(),
    KeyCode::Down => app.select_next(),
    _ => {}
  }
}

async fn handle_search_editing_mode(
  app: &mut App,
  key_code: KeyCode,
  app_arc: Arc<Mutex<App>>,
) {
  match key_code {
    KeyCode::Enter => {
      if !app.search_input.is_empty() {
        start_search(app, app_arc).await;
      }
      app.input_mode = InputMode::Normal;
    }
    KeyCode::Char(c) => app.search_input.push(c),
    KeyCode::Backspace => { app.search_input.pop(); }
    KeyCode::Esc => app.input_mode = InputMode::Normal,
    _ => {}
  }
}

async fn start_search(app: &mut App, app_arc: Arc<Mutex<App>>) {
  let name = app.search_input.clone();
  let settings = app.config.registries.clone();
  app.is_searching = true;

  let app_clone = Arc::clone(&app_arc);
  tokio::spawn(async move {
    let results = registry::check_all(&name, &settings).await;
    let mut app_guard = app_clone.lock().await;
    app_guard.search_results = results;
    app_guard.is_searching = false;
  });
}

/// Handle settings screen input
pub fn handle_settings_input(app: &mut App, key_code: KeyCode) {
  match key_code {
    KeyCode::Up => {
      if app.selected_setting > 0 {
        app.selected_setting -= 1;
      }
    }
    KeyCode::Down => {
      if app.selected_setting < app.registry_count() - 1 {
        app.selected_setting += 1;
      }
    }
    KeyCode::Enter | KeyCode::Char(' ') => {
      app.toggle_selected_registry();
    }
    _ => {}
  }
}

/// Handle register screen input
pub async fn handle_register_input(app: &mut App, key_code: KeyCode) {
  match key_code {
    KeyCode::Up => app.select_previous(),
    KeyCode::Down => app.select_next(),
    KeyCode::Enter => handle_registration(app).await,
    _ => {}
  }
}

async fn handle_registration(app: &mut App) {
  // Validate selection
  let available_registries = app.get_available_registries();
  if app.selected_registry >= available_registries.len() {
    app.register_status = Some("No registry selected".to_string());
    return;
  }

  let result = available_registries[app.selected_registry].clone();
  if result.available != Some(true) {
    app.register_status = Some("Name not available".to_string());
    return;
  }

  let token = match app.config.get_github_token() {
    Some(t) => t,
    None => {
      app.register_status = Some("Error: Set GITHUB_TOKEN environment variable".to_string());
      return;
    }
  };

  app.is_registering = true;
  let reg_result = execute_registration(&result.name, result.registry, &token).await;
  
  app.register_status = Some(match reg_result {
    RegistrationResult::Success(msg) => msg,
    RegistrationResult::Error(msg) => format!("Error: {}", msg),
  });
  app.is_registering = false;
}

async fn execute_registration(
  name: &str,
  registry_type: RegistryType,
  token: &str,
) -> RegistrationResult {
  match registry_type {
    RegistryType::GitHub => register_github(name, token).await,
    RegistryType::Npm => register_with_manifest(name, ManifestType::Npm, token).await,
    RegistryType::Crates => register_with_manifest(name, ManifestType::Crates, token).await,
    RegistryType::PyPi => register_with_manifest(name, ManifestType::PyPi, token).await,
    RegistryType::Brew => RegistrationResult::Success(
      "Homebrew: Create a formula and submit PR to homebrew-core".to_string()
    ),
    RegistryType::Flatpak => RegistrationResult::Success(
      "Flatpak: Submit your app to flathub.org/apps/submit".to_string()
    ),
    RegistryType::Debian => RegistrationResult::Success(
      "Debian: Follow ITP process at wiki.debian.org/ITP".to_string()
    ),
    RegistryType::DevDomain => RegistrationResult::Success(
      "Domain registration requires a registrar (e.g., Google Domains, Namecheap)".to_string()
    ),
  }
}

async fn register_github(name: &str, token: &str) -> RegistrationResult {
  match registry::github::create_repo(name, None, false, token).await {
    Ok(repo) => RegistrationResult::Success(format!("Created: {}", repo.html_url)),
    Err(e) => RegistrationResult::Error(format_github_error(e)),
  }
}

async fn register_with_manifest(
  name: &str,
  manifest_type: ManifestType,
  token: &str,
) -> RegistrationResult {
  match registry::github::create_repo_with_manifest(name, manifest_type, token).await {
    Ok(repo) => {
      let publish_cmd = match manifest_type {
        ManifestType::Npm => "npm publish",
        ManifestType::Crates => "cargo publish",
        ManifestType::PyPi => "twine upload",
      };
      RegistrationResult::Success(format!(
        "{} - Run '{}' to claim the name",
        repo.html_url, publish_cmd
      ))
    }
    Err(GitHubError::RepoExists) => {
      handle_existing_repo(name, manifest_type, token).await
    }
    Err(e) => RegistrationResult::Error(format_github_error(e)),
  }
}

async fn handle_existing_repo(
  name: &str,
  manifest_type: ManifestType,
  token: &str,
) -> RegistrationResult {
  let username = match registry::github::get_username(token).await {
    Ok(u) => u,
    Err(e) => return RegistrationResult::Error(format_github_error(e)),
  };

  match registry::github::add_manifest_if_missing(&username, name, manifest_type, token).await {
    Ok(true) => RegistrationResult::Success(format!(
      "Added {} to existing repo",
      manifest_type.filename()
    )),
    Ok(false) => RegistrationResult::Success(format!(
      "{} already exists in repo",
      manifest_type.filename()
    )),
    Err(e) => RegistrationResult::Error(format_github_error(e)),
  }
}

fn format_github_error(error: GitHubError) -> String {
  match error {
    GitHubError::AuthRequired => "Authentication required - check your token".to_string(),
    GitHubError::RepoExists => "Repository already exists".to_string(),
    GitHubError::InvalidName => "Invalid repository name".to_string(),
    GitHubError::RateLimited => "Rate limited - try again later".to_string(),
    GitHubError::ApiError(msg) => format!("API error: {}", msg),
    GitHubError::NetworkError(e) => format!("Network error: {}", e),
  }
}
