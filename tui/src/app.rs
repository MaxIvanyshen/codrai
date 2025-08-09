use crate::color::{ColorTheme, get_default_theme};

pub struct App {
    pub title: String,
    pub is_processing: bool,
    pub input: String,
    pub color_theme: Box<dyn ColorTheme>,
    pub mode: AppMode,
    codr: codr::Codr,
    pub messages: Vec<openai::Message>
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

impl App {
    pub fn new() -> App {
        App {
            title: "New Codr Session".to_string(),
            is_processing: false,
            input: String::new(),
            color_theme: get_default_theme(),
            mode: AppMode::Normal,
            codr: codr::Codr::new(),
            messages: Vec::new(),
        }
    }
}
