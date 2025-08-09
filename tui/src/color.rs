pub enum ColorType {
    Primary,
    Accent,
    Warning,
    Danger,
    Success,
}

pub trait ColorTheme {
    fn get_name(&self) -> String;
    fn get_color(&self, color_type: ColorType) -> Option<&'static u32>;
}

pub fn get_default_theme() -> Box<dyn ColorTheme> {
    Box::new(NordTheme::new())
}

// Struct for a Nord-inspired theme with fixed colors
pub struct NordTheme {
    primary: &'static u32,
    accent: &'static u32,
    warning: &'static u32,
    danger: &'static u32,
    success: &'static u32,
}

// Implement the Nord theme with approximate colors from the image
impl NordTheme {
    pub fn new() -> Self {
        NordTheme {
            primary: &0x3B4252,   // Darker gray
            accent: &0x88C0D0,   // Light blue
            warning: &0xEBCB8B, // Light yellow
            danger: &0xBF616A, // Light orange
            success: &0xA3BE8C,   // Light green
        }
    }
}

impl ColorTheme for NordTheme {
    fn get_name(&self) -> String {
        "Nord Theme".to_string()
    }
    fn get_color(&self, key: ColorType) -> Option<&'static u32> {
        match key {
            ColorType::Primary => Some(self.primary),
            ColorType::Accent => Some(self.accent),
            ColorType::Warning => Some(self.warning),
            ColorType::Danger => Some(self.danger),
            ColorType::Success => Some(self.success),
        }
    }
}
