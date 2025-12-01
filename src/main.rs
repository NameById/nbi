mod app;
mod cli;
mod config;
mod registry;
mod server;
mod ui;

use app::{App, InputMode, Screen};
use clap::Parser;
use cli::{Cli, Commands, PublishRegistry};
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use registry::RegistryType;
use std::{io, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();

  match cli.command {
    None | Some(Commands::Tui) => run_tui().await,
    Some(Commands::Serve { port, open }) => server::start(port, open).await,
    Some(Commands::Check { name, json }) => run_check(&name, json).await,
    Some(Commands::Domain { name, tlds, json }) => run_domain_check(&name, &tlds, json).await,
    Some(Commands::Publish { registry }) => run_publish(registry).await,
  }
}

async fn run_tui() -> anyhow::Result<()> {
  // Setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  // Create app state
  let app = Arc::new(Mutex::new(App::new()));

  // Run the app
  let res = run_app(&mut terminal, app).await;

  // Restore terminal
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  if let Err(err) = res {
    eprintln!("Error: {}", err);
  }

  Ok(())
}

async fn run_check(name: &str, json: bool) -> anyhow::Result<()> {
  let config = config::Config::load().unwrap_or_default();
  let results = registry::check_all(name, &config.registries).await;

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

async fn run_domain_check(name: &str, tlds: &str, json: bool) -> anyhow::Result<()> {
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
      results.push(registry::domain::check_full_domain(domain).await);
    }
    results
  } else {
    // Name + TLDs check
    let tld_list: Vec<&str> = tlds.split(',').map(|s| s.trim()).collect();
    registry::domain::check_multiple_tlds(name, &tld_list).await
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

async fn run_publish(registry: PublishRegistry) -> anyhow::Result<()> {
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

async fn run_app(
  terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
  app: Arc<Mutex<App>>,
) -> anyhow::Result<()> {
  loop {
    // Draw UI
    {
      let app_guard = app.lock().await;
      terminal.draw(|f| {
        ui::render(f, &app_guard);
        if app_guard.show_help {
          ui::render_help(f);
        }
      })?;
    }

    // Handle input with timeout for async operations
    if event::poll(Duration::from_millis(100))? {
      if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
          continue;
        }

        let mut app_guard = app.lock().await;

        // Global shortcuts
        match key.code {
          KeyCode::Char('q') if !matches!(app_guard.input_mode, InputMode::Editing) => {
            app_guard.should_quit = true;
          }
          KeyCode::Esc => {
            if app_guard.show_help {
              app_guard.show_help = false;
            } else if app_guard.input_mode == InputMode::Editing {
              app_guard.input_mode = InputMode::Normal;
            } else {
              app_guard.should_quit = true;
            }
          }
          KeyCode::Char('?') if app_guard.input_mode != InputMode::Editing => {
            app_guard.show_help = !app_guard.show_help;
          }
          KeyCode::Tab => {
            app_guard.toggle_screen();
          }
          KeyCode::Char('1') if app_guard.input_mode != InputMode::Editing => {
            app_guard.screen = Screen::Search;
          }
          KeyCode::Char('2') if app_guard.input_mode != InputMode::Editing => {
            app_guard.screen = Screen::Register;
          }
          KeyCode::Char('3') if app_guard.input_mode != InputMode::Editing => {
            app_guard.screen = Screen::Settings;
          }
          _ => {
            // Screen-specific handling
            match app_guard.screen {
              Screen::Search => {
                handle_search_input(&mut app_guard, key.code, Arc::clone(&app)).await;
              }
              Screen::Register => {
                handle_register_input(&mut app_guard, key.code).await;
              }
              Screen::Settings => {
                handle_settings_input(&mut app_guard, key.code);
              }
            }
          }
        }

        if app_guard.should_quit {
          break;
        }
      }
    }
  }

  Ok(())
}

async fn handle_search_input(app: &mut App, key: KeyCode, app_arc: Arc<Mutex<App>>) {
  // Disable input while searching
  if app.is_searching {
    return;
  }

  match app.input_mode {
    InputMode::Normal => match key {
      KeyCode::Char('i') | KeyCode::Char('e') | KeyCode::Enter => {
        app.input_mode = InputMode::Editing;
      }
      KeyCode::Up => app.select_previous(),
      KeyCode::Down => app.select_next(),
      _ => {}
    },
    InputMode::Editing => match key {
      KeyCode::Enter => {
        if !app.search_input.is_empty() {
          let name = app.search_input.clone();
          let settings = app.config.registries.clone();
          app.is_searching = true;

          // Spawn search in background
          let app_clone = Arc::clone(&app_arc);
          tokio::spawn(async move {
            let results = registry::check_all(&name, &settings).await;
            let mut app_guard = app_clone.lock().await;
            app_guard.search_results = results;
            app_guard.is_searching = false;
          });
        }
      }
      KeyCode::Char(c) => {
        app.search_input.push(c);
      }
      KeyCode::Backspace => {
        app.search_input.pop();
      }
      _ => {}
    },
  }
}

fn handle_settings_input(app: &mut App, key: KeyCode) {
  match key {
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

async fn handle_register_input(app: &mut App, key: KeyCode) {
  match key {
    KeyCode::Up => app.select_previous(),
    KeyCode::Down => app.select_next(),
    KeyCode::Enter => {
      // Extract needed values before mutable operations
      let selected_idx = app.selected_registry;
      let selected_registry = app
        .search_results
        .iter()
        .filter(|r| r.available == Some(true))
        .nth(selected_idx)
        .map(|r| r.registry);

      let token = app.config.get_github_token();
      let name = app.search_input.clone();

      if let Some(reg_type) = selected_registry {
        match reg_type {
          RegistryType::GitHub => {
            if let Some(token) = token {
              app.is_registering = true;
              app.register_status = Some("Creating GitHub repository...".to_string());

              match registry::github::create_repo(&name, None, false, &token).await {
                Ok(repo) => {
                  app.register_status = Some(format!("Success! Created: {}", repo.html_url));
                }
                Err(e) => {
                  app.register_status = Some(format!("Error: {}", e));
                }
              }
              app.is_registering = false;
            } else {
              app.register_status =
                Some("Error: Set GITHUB_TOKEN environment variable".to_string());
            }
          }
          RegistryType::Npm | RegistryType::Crates | RegistryType::PyPi => {
            if let Some(token) = token {
              app.is_registering = true;
              app.register_status = Some(format!(
                "Creating GitHub repo to reserve '{}' for {}...",
                name, reg_type
              ));

              let description = format!("Reserved package name for {}", reg_type);

              match registry::github::create_repo(&name, Some(&description), false, &token).await {
                Ok(repo) => {
                  app.register_status = Some(format!(
                    "Success! Repo created: {} - Now publish to {} to claim the name",
                    repo.html_url, reg_type
                  ));
                }
                Err(e) => {
                  app.register_status = Some(format!("Error: {}", e));
                }
              }
              app.is_registering = false;
            } else {
              app.register_status =
                Some("Error: Set GITHUB_TOKEN environment variable".to_string());
            }
          }
          RegistryType::Brew => {
            app.register_status = Some(
              "Homebrew: Create a formula and submit PR to homebrew-core".to_string(),
            );
          }
          RegistryType::Flatpak => {
            app.register_status =
              Some("Flatpak: Submit your app to flathub.org/apps/submit".to_string());
          }
          RegistryType::Debian => {
            app.register_status =
              Some("Debian: Follow ITP process at wiki.debian.org/ITP".to_string());
          }
          RegistryType::DevDomain => {
            app.register_status = Some(
              "Domain registration requires a registrar (e.g., Google Domains, Namecheap)"
                .to_string(),
            );
          }
        }
      }
    }
    _ => {}
  }
}
