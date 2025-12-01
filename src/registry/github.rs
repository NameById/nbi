use super::{AvailabilityResult, RegistryType};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};

const GITHUB_API_URL: &str = "https://api.github.com";

#[derive(Debug, Serialize)]
struct CreateRepoRequest {
  name: String,
  description: Option<String>,
  private: bool,
  auto_init: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RepoResponse {
  pub id: u64,
  pub name: String,
  pub full_name: String,
  pub html_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
  #[error("Authentication required: provide a GitHub personal access token")]
  AuthRequired,

  #[error("Repository name already exists")]
  RepoExists,

  #[error("Invalid repository name")]
  InvalidName,

  #[error("Rate limited")]
  RateLimited,

  #[error("API error: {0}")]
  ApiError(String),

  #[error("Network error: {0}")]
  NetworkError(#[from] reqwest::Error),
}

/// Check if a GitHub repository name is available for the authenticated user
///
/// API: GET https://api.github.com/repos/{owner}/{repo}
/// - 404: Repository not found (available)
/// - 200: Repository exists (not available)
#[allow(dead_code)]
pub async fn check_repo(owner: &str, name: &str, token: &str) -> AvailabilityResult {
  let url = format!("{}/repos/{}/{}", GITHUB_API_URL, owner, name);

  let client = reqwest::Client::new();
  match client
    .get(&url)
    .header(header::USER_AGENT, "nbi/0.1.0")
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .header(header::ACCEPT, "application/vnd.github+json")
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
        registry: RegistryType::GitHub,
        name: format!("{}/{}", owner, name),
        available,
        error: if available.is_none() {
          Some(format!("Unexpected status: {}", response.status()))
        } else {
          None
        },
      }
    }
    Err(e) => AvailabilityResult {
      registry: RegistryType::GitHub,
      name: format!("{}/{}", owner, name),
      available: None,
      error: Some(e.to_string()),
    },
  }
}

/// Create a new GitHub repository
///
/// API: POST https://api.github.com/user/repos
/// Required scope: public_repo (for public) or repo (for private)
pub async fn create_repo(
  name: &str,
  description: Option<&str>,
  private: bool,
  token: &str,
) -> Result<RepoResponse, GitHubError> {
  let url = format!("{}/user/repos", GITHUB_API_URL);

  let request = CreateRepoRequest {
    name: name.to_string(),
    description: description.map(String::from),
    private,
    auto_init: true, // Create with README to initialize
  };

  let client = reqwest::Client::new();
  let response = client
    .post(&url)
    .header(header::USER_AGENT, "nbi/0.1.0")
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .header(header::ACCEPT, "application/vnd.github+json")
    .json(&request)
    .send()
    .await?;

  match response.status() {
    StatusCode::CREATED => {
      let repo: RepoResponse = response.json().await?;
      Ok(repo)
    }
    StatusCode::UNAUTHORIZED => Err(GitHubError::AuthRequired),
    StatusCode::UNPROCESSABLE_ENTITY => {
      let body = response.text().await.unwrap_or_default();
      if body.contains("name already exists") {
        Err(GitHubError::RepoExists)
      } else {
        Err(GitHubError::InvalidName)
      }
    }
    StatusCode::FORBIDDEN => Err(GitHubError::RateLimited),
    _ => {
      let body = response.text().await.unwrap_or_default();
      Err(GitHubError::ApiError(body))
    }
  }
}

/// Get authenticated user's username
pub async fn get_username(token: &str) -> Result<String, GitHubError> {
  let url = format!("{}/user", GITHUB_API_URL);

  let client = reqwest::Client::new();
  let response = client
    .get(&url)
    .header(header::USER_AGENT, "nbi/0.1.0")
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .header(header::ACCEPT, "application/vnd.github+json")
    .send()
    .await?;

  if response.status() == StatusCode::UNAUTHORIZED {
    return Err(GitHubError::AuthRequired);
  }

  #[derive(Deserialize)]
  struct User {
    login: String,
  }

  let user: User = response.json().await?;
  Ok(user.login)
}

/// Registry type for manifest generation
#[derive(Debug, Clone, Copy)]
pub enum ManifestType {
  Npm,
  Crates,
  PyPi,
}

impl ManifestType {
  pub fn filename(&self) -> &'static str {
    match self {
      ManifestType::Npm => "package.json",
      ManifestType::Crates => "Cargo.toml",
      ManifestType::PyPi => "pyproject.toml",
    }
  }

  pub fn generate_content(&self, name: &str, description: &str) -> String {
    match self {
      ManifestType::Npm => format!(
        r#"{{
  "name": "{}",
  "version": "0.0.1",
  "description": "{}",
  "main": "index.js",
  "scripts": {{
    "test": "echo \"Error: no test specified\" && exit 1"
  }},
  "keywords": [],
  "author": "",
  "license": "MIT"
}}
"#,
        name, description
      ),
      ManifestType::Crates => format!(
        r#"[package]
name = "{}"
version = "0.0.1"
edition = "2021"
description = "{}"
license = "MIT"

[dependencies]
"#,
        name, description
      ),
      ManifestType::PyPi => format!(
        r#"[build-system]
requires = ["setuptools>=61.0"]
build-backend = "setuptools.build_meta"

[project]
name = "{}"
version = "0.0.1"
description = "{}"
readme = "README.md"
license = {{text = "MIT"}}
requires-python = ">=3.8"
classifiers = [
    "Programming Language :: Python :: 3",
    "License :: OSI Approved :: MIT License",
    "Operating System :: OS Independent",
]

[project.urls]
Homepage = "https://github.com/OWNER/{}"
"#,
        name, description, name
      ),
    }
  }
}

#[derive(Debug, Serialize)]
struct CreateFileRequest {
  message: String,
  content: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileContent {
  pub sha: String,
}

/// Check if a file exists in a repository
pub async fn check_file_exists(
  owner: &str,
  repo: &str,
  path: &str,
  token: &str,
) -> Result<Option<String>, GitHubError> {
  let url = format!("{}/repos/{}/{}/contents/{}", GITHUB_API_URL, owner, repo, path);

  let client = reqwest::Client::new();
  let response = client
    .get(&url)
    .header(header::USER_AGENT, "nbi/0.1.0")
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .header(header::ACCEPT, "application/vnd.github+json")
    .send()
    .await?;

  match response.status() {
    StatusCode::OK => {
      let file: FileContent = response.json().await?;
      Ok(Some(file.sha))
    }
    StatusCode::NOT_FOUND => Ok(None),
    StatusCode::UNAUTHORIZED => Err(GitHubError::AuthRequired),
    _ => {
      let body = response.text().await.unwrap_or_default();
      Err(GitHubError::ApiError(body))
    }
  }
}

/// Create or update a file in a repository
pub async fn create_or_update_file(
  owner: &str,
  repo: &str,
  path: &str,
  content: &str,
  message: &str,
  token: &str,
) -> Result<(), GitHubError> {
  use base64::{Engine as _, engine::general_purpose::STANDARD};
  
  let url = format!("{}/repos/{}/{}/contents/{}", GITHUB_API_URL, owner, repo, path);
  let encoded_content = STANDARD.encode(content);

  let request = CreateFileRequest {
    message: message.to_string(),
    content: encoded_content,
    branch: None,
  };

  let client = reqwest::Client::new();
  let response = client
    .put(&url)
    .header(header::USER_AGENT, "nbi/0.1.0")
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .header(header::ACCEPT, "application/vnd.github+json")
    .json(&request)
    .send()
    .await?;

  match response.status() {
    StatusCode::CREATED | StatusCode::OK => Ok(()),
    StatusCode::UNAUTHORIZED => Err(GitHubError::AuthRequired),
    StatusCode::UNPROCESSABLE_ENTITY => {
      let body = response.text().await.unwrap_or_default();
      Err(GitHubError::ApiError(format!("File operation failed: {}", body)))
    }
    _ => {
      let body = response.text().await.unwrap_or_default();
      Err(GitHubError::ApiError(body))
    }
  }
}

/// Create a repository with manifest file for the specified registry
pub async fn create_repo_with_manifest(
  name: &str,
  manifest_type: ManifestType,
  token: &str,
) -> Result<RepoResponse, GitHubError> {
  let description = format!("Reserved package name for {}", manifest_type.filename());
  
  // First create the repo
  let repo = create_repo(name, Some(&description), false, token).await?;
  
  // Get username for the owner
  let username = get_username(token).await?;
  
  // Wait a moment for GitHub to initialize the repo
  tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
  
  // Add manifest file
  let manifest_content = manifest_type.generate_content(name, &description);
  create_or_update_file(
    &username,
    name,
    manifest_type.filename(),
    &manifest_content,
    &format!("Add {} for package reservation", manifest_type.filename()),
    token,
  ).await?;
  
  Ok(repo)
}

/// Add manifest to existing repository if it doesn't exist
pub async fn add_manifest_if_missing(
  owner: &str,
  repo: &str,
  manifest_type: ManifestType,
  token: &str,
) -> Result<bool, GitHubError> {
  let filename = manifest_type.filename();
  
  // Check if file already exists
  if check_file_exists(owner, repo, filename, token).await?.is_some() {
    return Ok(false); // File already exists
  }
  
  // Create the manifest file
  let description = format!("Reserved package name for {}", filename);
  let content = manifest_type.generate_content(repo, &description);
  
  create_or_update_file(
    owner,
    repo,
    filename,
    &content,
    &format!("Add {} for package reservation", filename),
    token,
  ).await?;
  
  Ok(true) // File was created
}
