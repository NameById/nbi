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
#[allow(dead_code)]
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
