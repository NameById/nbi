use super::{AvailabilityResult, RegistryType};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

/// Check if a .dev domain is potentially available
///
/// This uses DNS lookup to check if the domain has any records.
/// Note: This is NOT a definitive availability check. A domain without
/// DNS records could still be registered but not configured.
/// For accurate results, you'd need a WHOIS API or registrar API.
///
/// Returns:
/// - available = Some(true): No DNS records found (might be available)
/// - available = Some(false): DNS records exist (definitely taken)
/// - available = None: Check failed
pub async fn check(name: &str) -> AvailabilityResult {
  let domain = format!("{}.dev", name);

  let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

  // Try to resolve A records
  match resolver.lookup_ip(&domain).await {
    Ok(_) => AvailabilityResult {
      registry: RegistryType::DevDomain,
      name: domain,
      available: Some(false), // Domain has DNS records, likely taken
      error: None,
    },
    Err(e) => {
      // Check if it's a "no records" error vs actual failure
      let err_str = e.to_string();
      let is_nxdomain = err_str.contains("no record")
        || err_str.contains("NXDOMAIN")
        || err_str.contains("NxDomain")
        || err_str.contains("no connections");

      if is_nxdomain {
        AvailabilityResult {
          registry: RegistryType::DevDomain,
          name: domain,
          available: Some(true), // No DNS records, might be available
          error: None,
        }
      } else {
        AvailabilityResult {
          registry: RegistryType::DevDomain,
          name: domain,
          available: None,
          error: Some(err_str),
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
