use crate::app::{App, InputMode};
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
      Constraint::Length(3), // Search input
      Constraint::Min(0),    // Results
    ])
    .split(area);

  render_search_input(frame, app, chunks[0]);
  render_results(frame, app, chunks[1]);
}

fn render_search_input(frame: &mut Frame, app: &App, area: Rect) {
  let (border_style, title) = match app.input_mode {
    InputMode::Normal => (
      Style::default().fg(Color::DarkGray),
      " Package Name (i/e to edit) ",
    ),
    InputMode::Editing => (
      Style::default().fg(Color::Yellow),
      " Package Name (Enter to search) ",
    ),
  };

  let input = Paragraph::new(app.search_input.as_str())
    .style(match app.input_mode {
      InputMode::Normal => Style::default(),
      InputMode::Editing => Style::default().fg(Color::Yellow),
    })
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style),
    );

  frame.render_widget(input, area);

  // Show cursor when editing
  if app.input_mode == InputMode::Editing {
    frame.set_cursor_position((
      area.x + app.search_input.len() as u16 + 1,
      area.y + 1,
    ));
  }
}

fn render_results(frame: &mut Frame, app: &App, area: Rect) {
  if app.search_results.is_empty() {
    let message = if app.is_searching {
      "Searching..."
    } else if app.search_input.is_empty() {
      "Enter a package name to check availability"
    } else {
      "Press Enter to search"
    };

    let placeholder = Paragraph::new(message)
      .style(Style::default().fg(Color::DarkGray))
      .block(Block::default().borders(Borders::ALL).title(" Results "));

    frame.render_widget(placeholder, area);
    return;
  }

  let items: Vec<ListItem> = app
    .search_results
    .iter()
    .map(|result| {
      let symbol = App::get_status_symbol(result);
      let color = App::get_status_color(result);

      let (status_text, error_text) = match (result.available, &result.error) {
        (Some(true), _) => ("Available", None),
        (Some(false), _) => ("Taken", None),
        (None, Some(err)) => {
          let short_err = if err.contains("timeout") || err.contains("Timeout") {
            "Timeout"
          } else if err.contains("rate") || err.contains("429") {
            "Rate Limited"
          } else if err.contains("403") || err.contains("Forbidden") {
            "Access Denied"
          } else if err.contains("connect") || err.contains("network") {
            "Network Error"
          } else if err.len() > 30 {
            "Error"
          } else {
            "Error"
          };
          (short_err, Some(err.as_str()))
        }
        (None, None) => ("Unknown", None),
      };

      let line = Line::from(vec![
        Span::styled(
          format!(" {} ", symbol),
          Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
          format!("{:<12}", result.registry),
          Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {:<14}", status_text), Style::default().fg(color)),
        if let Some(err) = error_text {
          let truncated = if err.len() > 40 {
            format!("{}...", &err[..40])
          } else {
            err.to_string()
          };
          Span::styled(format!("({})", truncated), Style::default().fg(Color::Red))
        } else {
          Span::raw("")
        },
      ]);

      ListItem::new(line)
    })
    .collect();

  let results_list = List::new(items).block(
    Block::default()
      .borders(Borders::ALL)
      .title(format!(" Results for '{}' ", app.search_input)),
  );

  frame.render_widget(results_list, area);
}
