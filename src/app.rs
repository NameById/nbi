use crate::config::Config;
use crate::registry::AvailabilityResult;

/// Current screen/view in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
  Search,
  Register,
  Settings,
}

/// Application state
pub struct App {
  pub config: Config,
  pub screen: Screen,
  pub should_quit: bool,

  // Search state
  pub search_input: String,
  pub search_results: Vec<AvailabilityResult>,
  pub is_searching: bool,

  // Register state
  pub selected_registry: usize,
  pub register_status: Option<String>,
  pub is_registering: bool,

  // Settings state
  pub selected_setting: usize,

  // UI state
  pub show_help: bool,
  pub input_mode: InputMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
  Normal,
  Editing,
}

impl App {
  pub fn new() -> Self {
    let config = Config::load().unwrap_or_default();

    Self {
      config,
      screen: Screen::Search,
      should_quit: false,

      search_input: String::new(),
      search_results: Vec::new(),
      is_searching: false,

      selected_registry: 0,
      register_status: None,
      is_registering: false,

      selected_setting: 0,

      show_help: false,
      input_mode: InputMode::Editing,
    }
  }

  /// Save current config
  pub fn save_config(&self) -> anyhow::Result<()> {
    self.config.save()
  }

  /// Get available registries from search results
  pub fn get_available_registries(&self) -> Vec<&AvailabilityResult> {
    self.search_results
      .iter()
      .filter(|r| r.available == Some(true))
      .collect()
  }

  /// Toggle screen between Search, Register, and Settings
  pub fn toggle_screen(&mut self) {
    self.screen = match self.screen {
      Screen::Search => Screen::Register,
      Screen::Register => Screen::Settings,
      Screen::Settings => Screen::Search,
    };
  }

  /// Get number of registry settings
  pub fn registry_count(&self) -> usize {
    7 // npm, crates, pypi, brew, flatpak, debian, dev_domain
  }

  /// Toggle registry at current selection
  pub fn toggle_selected_registry(&mut self) {
    match self.selected_setting {
      0 => self.config.registries.npm = !self.config.registries.npm,
      1 => self.config.registries.crates = !self.config.registries.crates,
      2 => self.config.registries.pypi = !self.config.registries.pypi,
      3 => self.config.registries.brew = !self.config.registries.brew,
      4 => self.config.registries.flatpak = !self.config.registries.flatpak,
      5 => self.config.registries.debian = !self.config.registries.debian,
      6 => self.config.registries.dev_domain = !self.config.registries.dev_domain,
      _ => {}
    }
    // Auto-save config
    let _ = self.save_config();
  }

  /// Move selection up in register screen
  pub fn select_previous(&mut self) {
    let available_count = self.get_available_registries().len();
    if available_count > 0 && self.selected_registry > 0 {
      self.selected_registry -= 1;
    }
  }

  /// Move selection down in register screen
  pub fn select_next(&mut self) {
    let available_count = self.get_available_registries().len();
    if available_count > 0 && self.selected_registry < available_count - 1 {
      self.selected_registry += 1;
    }
  }

  /// Get status text for a registry result
  pub fn get_status_symbol(result: &AvailabilityResult) -> &'static str {
    match result.available {
      Some(true) => "✓",
      Some(false) => "✗",
      None => "?",
    }
  }

  /// Get status color for a registry result
  pub fn get_status_color(result: &AvailabilityResult) -> ratatui::style::Color {
    use ratatui::style::Color;
    match result.available {
      Some(true) => Color::Green,
      Some(false) => Color::Red,
      None => Color::Yellow,
    }
  }
}

impl Default for App {
  fn default() -> Self {
    Self::new()
  }
}
