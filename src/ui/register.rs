use crate::app::App;
use crate::registry::RegistryType;
use ratatui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, List, ListItem, Paragraph},
  Frame,
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Info
      Constraint::Min(0),    // Registry list
      Constraint::Length(3), // Status
    ])
    .split(area);

  render_info(frame, app, chunks[0]);
  render_registry_list(frame, app, chunks[1]);
  render_status(frame, app, chunks[2]);
}

fn render_info(frame: &mut Frame, app: &App, area: Rect) {
  let has_token = app.config.get_github_token().is_some();
  let token_status = if has_token {
    Span::styled("✓ GitHub token configured", Style::default().fg(Color::Green))
  } else {
    Span::styled(
      "✗ GitHub token not set (export GITHUB_TOKEN or add to config)",
      Style::default().fg(Color::Red),
    )
  };

  let info = Paragraph::new(Line::from(vec![Span::raw("  "), token_status]))
    .block(Block::default().borders(Borders::ALL).title(" Configuration "));

  frame.render_widget(info, area);
}

fn render_registry_list(frame: &mut Frame, app: &App, area: Rect) {
  let available = app.get_available_registries();

  if available.is_empty() {
    let message = if app.search_results.is_empty() {
      "Search for a package name first (Tab to switch to Search)"
    } else {
      "No available registries found for this name"
    };

    let placeholder = Paragraph::new(message)
      .style(Style::default().fg(Color::DarkGray))
      .block(
        Block::default()
          .borders(Borders::ALL)
          .title(" Available Registries "),
      );

    frame.render_widget(placeholder, area);
    return;
  }

  let items: Vec<ListItem> = available
    .iter()
    .enumerate()
    .map(|(i, result)| {
      let is_selected = i == app.selected_registry;
      let prefix = if is_selected { "▶ " } else { "  " };

      let style = if is_selected {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
      } else {
        Style::default()
      };

      let action = match result.registry {
        RegistryType::GitHub => "Create repository",
        RegistryType::Npm => "Reserve via GitHub",
        RegistryType::Crates => "Reserve via GitHub",
        RegistryType::PyPi => "Reserve via GitHub",
        RegistryType::DevDomain => "Check registrar",
      };

      let line = Line::from(vec![
        Span::styled(prefix, style),
        Span::styled(format!("{:<12}", result.registry), style),
        Span::styled(format!(" - {}", action), Style::default().fg(Color::DarkGray)),
      ]);

      ListItem::new(line)
    })
    .collect();

  let list = List::new(items).block(
    Block::default()
      .borders(Borders::ALL)
      .title(" Available Registries (↑/↓ to select, Enter to register) "),
  );

  frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
  let status_text = if let Some(ref status) = app.register_status {
    status.as_str()
  } else if app.is_registering {
    "Registering..."
  } else {
    "Select a registry and press Enter to register"
  };

  let style = if app.register_status.as_ref().is_some_and(|s| s.contains("Error")) {
    Style::default().fg(Color::Red)
  } else if app.register_status.as_ref().is_some_and(|s| s.contains("Success")) {
    Style::default().fg(Color::Green)
  } else {
    Style::default().fg(Color::DarkGray)
  };

  let status = Paragraph::new(status_text)
    .style(style)
    .block(Block::default().borders(Borders::ALL).title(" Status "));

  frame.render_widget(status, area);
}
