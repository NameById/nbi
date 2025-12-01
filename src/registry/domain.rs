use super::{AvailabilityResult, RegistryType};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

/// Check if a .dev domain is potentially available
///
/// Uses DNS lookup to check if the domain has any A records
pub async fn check(name: &str) -> AvailabilityResult {
  check_tld(name, "dev").await
}

/// Check if a domain with specific TLD is available
pub async fn check_tld(name: &str, tld: &str) -> AvailabilityResult {
  let domain = format!("{}.{}", name, tld);

  let resolver =
    TokioAsyncResolver::tokio(ResolverConfig::google(), ResolverOpts::default());

  match resolver.lookup_ip(&domain).await {
    Ok(response) => {
      // If we get IP addresses, domain is taken (not available)
      let has_records = response.iter().count() > 0;
      AvailabilityResult {
        registry: RegistryType::DevDomain,
        name: domain,
        available: Some(!has_records),
        error: None,
      }
    }
    Err(e) => {
      // NXDOMAIN means the domain doesn't exist (available)
      let error_str = e.to_string();
      if error_str.contains("NXDOMAIN") || error_str.contains("no record") {
        AvailabilityResult {
          registry: RegistryType::DevDomain,
          name: domain,
          available: Some(true),
          error: None,
        }
      } else {
        AvailabilityResult {
          registry: RegistryType::DevDomain,
          name: domain,
          available: None,
          error: Some(error_str),
        }
      }
    }
  }
}

/// Check multiple TLDs at once
pub async fn check_multiple_tlds(name: &str, tlds: &[&str]) -> Vec<AvailabilityResult> {
  let futures: Vec<_> = tlds.iter().map(|tld| check_tld(name, tld)).collect();
  futures::future::join_all(futures).await
}

/// Check a full domain (e.g., "banana.wiki")
pub async fn check_full_domain(domain: &str) -> AvailabilityResult {
  let resolver =
    TokioAsyncResolver::tokio(ResolverConfig::google(), ResolverOpts::default());

  match resolver.lookup_ip(domain).await {
    Ok(response) => {
      // If we get IP addresses, domain is taken (not available)
      let has_records = response.iter().count() > 0;
      AvailabilityResult {
        registry: RegistryType::DevDomain,
        name: domain.to_string(),
        available: Some(!has_records),
        error: None,
      }
    }
    Err(e) => {
      let error_str = e.to_string();
      if error_str.contains("NXDOMAIN") || error_str.contains("no record") {
        AvailabilityResult {
          registry: RegistryType::DevDomain,
          name: domain.to_string(),
          available: Some(true),
          error: None,
        }
      } else {
        AvailabilityResult {
          registry: RegistryType::DevDomain,
          name: domain.to_string(),
          available: None,
          error: Some(error_str),
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_check_existing_domain() {
    // google.dev should exist
    let result = check("google").await;
    assert_eq!(result.available, Some(false));
  }
}
