use anyhow::Result;
use crate::cli::{PublishRegistry};

pub async fn run_check(name: &str, json: bool) -> Result<()> {
  let config = crate::config::Config::load()?;
  let results = crate::registry::check_all(name, &config.registries).await;

  if json {
    println!("{}", serde_json::to_string_pretty(&results)?);
  } else {
    println!("Checking availability for: {}\n", name);
    for r in &results {
      let status = match r.available {
        Some(true) => "\x1b[32m✓ Available\x1b[0m",
        Some(false) => "\x1b[31m✗ Taken\x1b[0m",
        None => "\x1b[33m? Unknown\x1b[0m",
      };
      print!("  {:<12} {}", r.registry.to_string(), status);
      if let Some(ref err) = r.error {
        print!(" ({})", err);
      }
      println!();
    }
  }
  Ok(())
}

pub async fn run_domain_check(name: &str, tlds: &str, json: bool) -> Result<()> {
  // Check if input is a full domain (contains a dot)
  let results = if name.contains('.') {
    // Full domain check - also check additional TLDs if specified
    let mut domains = vec![name.to_string()];
    
    // Parse the base name and add other TLDs
    if let Some(dot_pos) = name.rfind('.') {
      let base = &name[..dot_pos];
      for tld in tlds.split(',').map(|s| s.trim()) {
        let domain = format!("{}.{}", base, tld);
        if domain != name {
          domains.push(domain);
        }
      }
    }
    
    let mut results = Vec::new();
    for domain in &domains {
      results.push(crate::registry::domain::check_full_domain(domain).await);
    }
    results
  } else {
    // Name + TLDs check
    let tld_list: Vec<&str> = tlds.split(',').map(|s| s.trim()).collect();
    crate::registry::domain::check_multiple_tlds(name, &tld_list).await
  };

  if json {
    println!("{}", serde_json::to_string_pretty(&results)?);
  } else {
    println!("Checking domain availability for: {}\n", name);
    for r in &results {
      let status = match r.available {
        Some(true) => "\x1b[32m✓ Available\x1b[0m",
        Some(false) => "\x1b[31m✗ Taken\x1b[0m",
        None => "\x1b[33m? Unknown\x1b[0m",
      };
      println!("  {:<25} {}", r.name, status);
    }
  }
  Ok(())
}

pub async fn run_publish(registry: PublishRegistry) -> Result<()> {
  match registry {
    PublishRegistry::Npm { path } => {
      println!("Publishing to npm from: {}", path);
      let status = std::process::Command::new("npm")
        .args(["publish"])
        .current_dir(&path)
        .status()?;
      if !status.success() {
        anyhow::bail!("npm publish failed");
      }
    }
    PublishRegistry::Crates { path } => {
      println!("Publishing to crates.io from: {}", path);
      let status = std::process::Command::new("cargo")
        .args(["publish"])
        .current_dir(&path)
        .status()?;
      if !status.success() {
        anyhow::bail!("cargo publish failed");
      }
    }
    PublishRegistry::Pypi { path } => {
      println!("Publishing to PyPI from: {}", path);
      // Build
      let build = std::process::Command::new("python")
        .args(["-m", "build"])
        .current_dir(&path)
        .status()?;
      if !build.success() {
        anyhow::bail!("python build failed");
      }
      // Upload
      let upload = std::process::Command::new("python")
        .args(["-m", "twine", "upload", "dist/*"])
        .current_dir(&path)
        .status()?;
      if !upload.success() {
        anyhow::bail!("twine upload failed");
      }
    }
  }
  println!("✓ Published successfully!");
  Ok(())
}
