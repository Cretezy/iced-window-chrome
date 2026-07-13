use iced::Color;
use std::hash::{Hash, Hasher};

/// Cross-platform chrome settings. Each platform reads only its own section.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChromeSettings {
    pub windows: WindowsChromeSettings,
    pub macos: MacosChromeSettings,
    pub linux: LinuxChromeSettings,
}

impl Hash for ChromeSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.windows.hash(state);
        self.macos.hash(state);
        self.linux.hash(state);
    }
}

/// Windows caption button visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CaptionButtons {
    pub close: bool,
    pub minimize: bool,
    pub maximize: bool,
}

impl Default for CaptionButtons {
    fn default() -> Self {
        Self {
            close: true,
            minimize: true,
            maximize: true,
        }
    }
}

/// Windows 11 corner preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowCornerPreference {
    Default,
    DoNotRound,
    Round,
    RoundSmall,
}

/// System-managed Windows backdrop material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowsBackdrop {
    None,
    Mica,
    Acrylic,
    MicaAlt,
}

/// Native macOS titlebar separator style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacosTitlebarSeparatorStyle {
    Automatic,
    None,
    Line,
    Shadow,
}

/// Native Linux/X11 window manager chrome settings.
#[derive(Debug, Clone, PartialEq)]
pub struct LinuxChromeSettings {
    pub decorations: bool,
    pub buttons: CaptionButtons,
}

impl Default for LinuxChromeSettings {
    fn default() -> Self {
        Self {
            decorations: true,
            buttons: CaptionButtons::default(),
        }
    }
}

impl Hash for LinuxChromeSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.decorations.hash(state);
        self.buttons.hash(state);
    }
}

/// Native Windows window chrome settings.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowsChromeSettings {
    pub caption: bool,
    pub border: bool,
    pub buttons: CaptionButtons,
    pub corner_preference: Option<WindowCornerPreference>,
    pub border_color: Option<Color>,
    pub title_background_color: Option<Color>,
    pub title_text_color: Option<Color>,
    pub backdrop: Option<WindowsBackdrop>,
}

impl Default for WindowsChromeSettings {
    fn default() -> Self {
        Self {
            caption: true,
            border: true,
            buttons: CaptionButtons::default(),
            corner_preference: None,
            border_color: None,
            title_background_color: None,
            title_text_color: None,
            backdrop: None,
        }
    }
}

impl Hash for WindowsChromeSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.caption.hash(state);
        self.border.hash(state);
        self.buttons.hash(state);
        self.corner_preference.hash(state);
        hash_color(self.border_color, state);
        hash_color(self.title_background_color, state);
        hash_color(self.title_text_color, state);
        self.backdrop.hash(state);
    }
}

/// Native macOS titlebar settings.
#[derive(Debug, Clone, PartialEq)]
pub struct MacosChromeSettings {
    pub titlebar: bool,
    pub title: bool,
    pub traffic_lights: bool,
    pub titlebar_transparent: bool,
    pub fullsize_content_view: bool,
    pub titlebar_height: Option<f64>,
    pub traffic_light_offset_x: Option<f64>,
    pub traffic_light_offset_y: Option<f64>,
    pub titlebar_separator_style: Option<MacosTitlebarSeparatorStyle>,
}

impl Default for MacosChromeSettings {
    fn default() -> Self {
        Self {
            titlebar: true,
            title: true,
            traffic_lights: true,
            titlebar_transparent: false,
            fullsize_content_view: false,
            titlebar_height: None,
            traffic_light_offset_x: None,
            traffic_light_offset_y: None,
            titlebar_separator_style: None,
        }
    }
}

impl Hash for MacosChromeSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.titlebar.hash(state);
        self.title.hash(state);
        self.traffic_lights.hash(state);
        self.titlebar_transparent.hash(state);
        self.fullsize_content_view.hash(state);
        hash_f64(self.titlebar_height, state);
        hash_f64(self.traffic_light_offset_x, state);
        hash_f64(self.traffic_light_offset_y, state);
        self.titlebar_separator_style.hash(state);
    }
}

fn hash_color<H: Hasher>(color: Option<Color>, state: &mut H) {
    match color {
        Some(color) => {
            true.hash(state);
            color.r.to_bits().hash(state);
            color.g.to_bits().hash(state);
            color.b.to_bits().hash(state);
            color.a.to_bits().hash(state);
        }
        None => false.hash(state),
    }
}

fn hash_f64<H: Hasher>(value: Option<f64>, state: &mut H) {
    match value {
        Some(value) => {
            true.hash(state);
            value.to_bits().hash(state);
        }
        None => false.hash(state),
    }
}
