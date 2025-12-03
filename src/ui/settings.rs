use crate::app::App;
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
      Constraint::Length(3), // Title
      Constraint::Min(0),    // Registry list
      Constraint::Length(3), // Help
    ])
    .split(area);

  render_title(frame, chunks[0]);
  render_registry_list(frame, app, chunks[1]);
  render_help(frame, chunks[2]);
}

fn render_title(frame: &mut Frame, area: Rect) {
  let title = Paragraph::new("Toggle registries to include in search")
    .style(Style::default().fg(Color::Cyan))
    .block(Block::default().borders(Borders::ALL).title(" Settings "));

  frame.render_widget(title, area);
}

fn render_registry_list(frame: &mut Frame, app: &App, area: Rect) {
  let registries = [
    ("npm", app.config.registries.npm, "npmjs.com"),
    ("crates.io", app.config.registries.crates, "crates.io"),
    ("PyPI", app.config.registries.pypi, "pypi.org"),
    ("GitHub", app.config.registries.github, "github.com/user"),
    ("Homebrew", app.config.registries.brew, "brew.sh"),
    ("Flatpak", app.config.registries.flatpak, "flathub.org"),
    ("Debian", app.config.registries.debian, "debian.org"),
    (".dev Domain", app.config.registries.dev_domain, "DNS lookup"),
  ];

  let items: Vec<ListItem> = registries
    .iter()
    .enumerate()
    .map(|(i, (name, enabled, desc))| {
      let is_selected = i == app.selected_setting;
      let prefix = if is_selected { "▶ " } else { "  " };

      let checkbox = if *enabled { "[✓]" } else { "[ ]" };
      let checkbox_color = if *enabled { Color::Green } else { Color::DarkGray };

      let style = if is_selected {
        Style::default().add_modifier(Modifier::BOLD)
      } else {
        Style::default()
      };

      let line = Line::from(vec![
        Span::styled(prefix, style),
        Span::styled(checkbox, Style::default().fg(checkbox_color)),
        Span::styled(format!(" {:<12}", name), style),
        Span::styled(format!(" - {}", desc), Style::default().fg(Color::DarkGray)),
      ]);

      ListItem::new(line)
    })
    .collect();

  let list = List::new(items).block(
    Block::default()
      .borders(Borders::ALL)
      .title(" Registries "),
  );

  frame.render_widget(list, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
  let help = Paragraph::new("↑/↓ Navigate | Enter/Space Toggle | Tab Switch screen")
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL));

  frame.render_widget(help, area);
}
