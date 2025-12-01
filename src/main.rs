mod app;
mod config;
mod registry;
mod ui;

use app::{App, InputMode, Screen};
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
          _ => {
            // Screen-specific handling
            match app_guard.screen {
              Screen::Search => {
                handle_search_input(&mut app_guard, key.code, Arc::clone(&app)).await;
              }
              Screen::Register => {
                handle_register_input(&mut app_guard, key.code).await;
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
          app.is_searching = true;

          // Spawn search in background
          let app_clone = Arc::clone(&app_arc);
          tokio::spawn(async move {
            let results = registry::check_all(&name).await;
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
