use crate::color::{ColorTheme, get_default_theme};
use tui_textarea::TextArea;
use ratatui::widgets::ListState;

pub struct App<'a> {
    pub title: String,
    pub is_processing: bool,
    pub color_theme: Box<dyn ColorTheme>,
    pub mode: AppMode,
    pub input: &'a mut TextArea<'a>,
    codr: codr::Codr,
    pub messages: Vec<openai::Message>,
    pub messages_state: ListState,
}

pub enum AppMode {
    Normal,
    Processing,
}

impl PartialEq for AppMode {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (AppMode::Normal, AppMode::Normal) | (AppMode::Processing, AppMode::Processing))
    }
}

impl AppMode {
    pub fn to_string(&self) -> &str {
        match self {
            AppMode::Normal => "NORMAL",
            AppMode::Processing => "PROCESSING",
        }
    }
}

impl<'a> App<'a> {
    pub fn new(input: &'a mut TextArea<'a>) -> App<'a> {
        App {
            title: "New Codr Session".to_string(),
            is_processing: false,
            input,
            color_theme: get_default_theme(),
            mode: AppMode::Normal,
            codr: codr::Codr::new(),
            messages: Vec::new(),
            messages_state: ListState::default(),
        }
    }

    pub fn clear_input(&mut self) {
        self.input.select_all();
        self.input.delete_char();
    }

    pub fn next_message(&mut self) {
        let i = match self.messages_state.selected() {
            Some(i) if i >= self.messages.len() - 1 => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.messages_state.select(Some(i));
    }

    pub fn previous_message(&mut self) {
        let i = match self.messages_state.selected() {
            Some(i) if i == 0 => self.messages.len().saturating_sub(1),
            Some(i) => i - 1,
            None => 0,
        };
        self.messages_state.select(Some(i));
    }
}
