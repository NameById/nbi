use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nbi")]
#[command(about = "Check package name availability across registries", long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
  /// Start TUI mode (default)
  Tui,

  /// Start web server for GUI
  Serve {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Open browser automatically
    #[arg(short, long)]
    open: bool,
  },

  /// Check name availability (CLI mode)
  Check {
    /// Package name to check
    name: String,

    /// Output as JSON
    #[arg(short, long)]
    json: bool,
  },

  /// Check domain availability
  Domain {
    /// Domain name (e.g., example.com)
    name: String,

    /// TLDs to check (comma-separated, default: com,net,org,io,dev)
    #[arg(short, long, default_value = "com,net,org,io,dev")]
    tlds: String,

    /// Output as JSON
    #[arg(short, long)]
    json: bool,
  },

  /// Publish package to registry
  Publish {
    #[command(subcommand)]
    registry: PublishRegistry,
  },
}

#[derive(Subcommand)]
pub enum PublishRegistry {
  /// Publish to npm
  Npm {
    /// Package directory
    #[arg(default_value = ".")]
    path: String,
  },

  /// Publish to crates.io
  Crates {
    /// Package directory
    #[arg(default_value = ".")]
    path: String,
  },

  /// Publish to PyPI
  Pypi {
    /// Package directory
    #[arg(default_value = ".")]
    path: String,
  },
}
