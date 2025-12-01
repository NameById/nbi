pub mod register;
pub mod search;
pub mod settings;

use crate::app::{App, InputMode, Screen};
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Paragraph, Tabs},
  Frame,
};

/// Render the main UI
pub fn render(frame: &mut Frame, app: &App) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Tabs
      Constraint::Min(0),    // Content
      Constraint::Length(1), // Status bar
    ])
    .split(frame.area());

  render_tabs(frame, app, chunks[0]);

  match app.screen {
    Screen::Search => search::render(frame, app, chunks[1]),
    Screen::Register => register::render(frame, app, chunks[1]),
    Screen::Settings => settings::render(frame, app, chunks[1]),
  }

  render_status_bar(frame, app, chunks[2]);
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
  let titles = vec!["Search [1]", "Register [2]", "Settings [3]"];
  let selected = match app.screen {
    Screen::Search => 0,
    Screen::Register => 1,
    Screen::Settings => 2,
  };

  let tabs = Tabs::new(titles)
    .block(Block::default().borders(Borders::ALL).title(" nbi "))
    .select(selected)
    .style(Style::default().fg(Color::White))
    .highlight_style(
      Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD),
    );

  frame.render_widget(tabs, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
  let (msg, style) = if app.is_searching {
    ("Searching...".to_string(), Style::default().fg(Color::Yellow))
  } else if app.is_registering {
    ("Registering...".to_string(), Style::default().fg(Color::Yellow))
  } else {
    // Check for errors in search results
    let error_count = app
      .search_results
      .iter()
      .filter(|r| r.error.is_some())
      .count();

    if error_count > 0 && app.screen == Screen::Search {
      (
        format!("{} error(s) occurred. Check results for details.", error_count),
        Style::default().fg(Color::Red),
      )
    } else {
      let mode_hint = match (app.screen, app.input_mode) {
        (Screen::Search, InputMode::Normal) => "NORMAL | i,e to edit | Enter to focus",
        (Screen::Search, InputMode::Editing) => "EDITING | Esc to unfocus | Enter to search",
        (Screen::Register, _) => "↑/↓ select | Enter to register | ? help",
        (Screen::Settings, _) => "↑/↓ select | Enter/Space toggle | ? help",
      };
      (mode_hint.to_string(), Style::default().fg(Color::DarkGray))
    }
  };

  let status = Paragraph::new(msg).style(style);
  frame.render_widget(status, area);
}

/// Render help popup
pub fn render_help(frame: &mut Frame) {
  let area = centered_rect(60, 70, frame.area());

  let help_text = vec![
    Line::from(Span::styled(
      "Keyboard Shortcuts",
      Style::default().add_modifier(Modifier::BOLD),
    )),
    Line::from(""),
    Line::from("  q          - Quit (in Normal mode)"),
    Line::from("  Esc        - Unfocus input / Close popup / Quit"),
    Line::from("  1          - Go to Search screen"),
    Line::from("  2          - Go to Register screen"),
    Line::from("  Tab        - Switch between screens"),
    Line::from("  ?          - Toggle this help"),
    Line::from(""),
    Line::from(Span::styled(
      "Search Screen",
      Style::default().add_modifier(Modifier::BOLD),
    )),
    Line::from("  i, e       - Enter edit mode (focus input)"),
    Line::from("  Enter      - Focus input / Execute search"),
    Line::from("  Esc        - Exit edit mode (unfocus input)"),
    Line::from(""),
    Line::from(Span::styled(
      "Register Screen",
      Style::default().add_modifier(Modifier::BOLD),
    )),
    Line::from("  ↑/↓        - Navigate available registries"),
    Line::from("  Enter      - Register selected"),
    Line::from(""),
    Line::from(Span::styled(
      "Note",
      Style::default().fg(Color::Yellow),
    )),
    Line::from("  GitHub token required for registration"),
    Line::from("  Set GITHUB_TOKEN env or add to config"),
  ];

  let help = Paragraph::new(help_text)
    .block(Block::default().borders(Borders::ALL).title(" Help "))
    .style(Style::default().bg(Color::DarkGray));

  frame.render_widget(ratatui::widgets::Clear, area);
  frame.render_widget(help, area);
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage((100 - percent_x) / 2),
      Constraint::Percentage(percent_x),
      Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
