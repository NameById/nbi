mod app;
mod cli;
mod cli_commands;
mod config;
mod registry;
mod server;
mod tui;
mod ui;

use clap::Parser;
use cli::{Cli, Commands};

use cli_commands::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();

  match cli.command {
    None | Some(Commands::Tui) => tui::TuiRunner::run().await,
    Some(Commands::Serve { port, open }) => server::start(port, open).await,
    Some(Commands::Check { name, json }) => run_check(&name, json).await,
    Some(Commands::Domain { name, tlds, json }) => run_domain_check(&name, &tlds, json).await,
    Some(Commands::Publish { registry }) => run_publish(registry).await,
  }
}

