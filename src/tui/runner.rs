//! TUI runner with clean event loop architecture

use crate::app::{App, InputMode, Screen};
use crate::tui::handlers;
use crate::ui;
use anyhow::Result;
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, sync::Arc, time::Duration};
use tokio::sync::Mutex;

const POLL_TIMEOUT_MS: u64 = 100;

pub struct TuiRunner;

impl TuiRunner {
  pub async fn run() -> Result<()> {
    let mut terminal = Self::setup_terminal()?;
    let app = Arc::new(Mutex::new(App::new()));
    
    let res = Self::run_event_loop(&mut terminal, app).await;
    
    Self::restore_terminal()?;
    res
  }

  fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
  }

  fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
  }

  async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: Arc<Mutex<App>>,
  ) -> Result<()> {
    loop {
      // Render UI
      {
        let app_guard = app.lock().await;
        if app_guard.should_quit {
          break;
        }
        terminal.draw(|f| {
          ui::render(f, &app_guard);
          if app_guard.show_help {
            ui::render_help(f);
          }
        })?;
      }

      // Handle events
      if event::poll(Duration::from_millis(POLL_TIMEOUT_MS))? {
        if let Event::Key(key) = event::read()? {
          if key.kind != KeyEventKind::Press {
            continue;
          }
          Self::handle_key_event(&app, key.code).await?;
        }
      }
    }
    Ok(())
  }

  async fn handle_key_event(app: &Arc<Mutex<App>>, key_code: KeyCode) -> Result<()> {
    let mut app_guard = app.lock().await;
    let is_editing = app_guard.input_mode == InputMode::Editing;
    let is_busy = app_guard.is_searching || app_guard.is_registering;

    // Allow quit even when busy
    if key_code == KeyCode::Esc && is_busy {
      return Ok(()); // Ignore ESC during operations
    }

    // Global shortcuts (available in non-editing mode)
    match key_code {
      KeyCode::Char('q') if !is_editing => {
        app_guard.should_quit = true;
        return Ok(());
      }
      KeyCode::Esc => {
        if app_guard.show_help {
          app_guard.show_help = false;
        } else if is_editing {
          app_guard.input_mode = InputMode::Normal;
        } else {
          app_guard.should_quit = true;
        }
        return Ok(());
      }
      KeyCode::Char('?') if !is_editing => {
        app_guard.show_help = !app_guard.show_help;
        return Ok(());
      }
      KeyCode::Tab if !is_editing => {
        app_guard.toggle_screen();
        return Ok(());
      }
      KeyCode::Char('1') if !is_editing => {
        app_guard.screen = Screen::Search;
        return Ok(());
      }
      KeyCode::Char('2') if !is_editing => {
        app_guard.screen = Screen::Register;
        return Ok(());
      }
      KeyCode::Char('3') if !is_editing => {
        app_guard.screen = Screen::Settings;
        return Ok(());
      }
      _ => {}
    }

    // Block screen-specific actions when busy
    if is_busy {
      return Ok(());
    }

    // Screen-specific handling
    let current_screen = app_guard.screen;
    drop(app_guard);

    match current_screen {
      Screen::Search => {
        let mut guard = app.lock().await;
        handlers::handle_search_input(&mut guard, key_code, Arc::clone(app)).await;
      }
      Screen::Register => {
        let mut guard = app.lock().await;
        handlers::handle_register_input(&mut guard, key_code).await;
      }
      Screen::Settings => {
        let mut guard = app.lock().await;
        handlers::handle_settings_input(&mut guard, key_code);
      }
    }

    Ok(())
  }
}
