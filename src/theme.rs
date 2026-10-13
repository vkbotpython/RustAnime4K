//! Color themes.
//!
//! The 9 palettes become `egui::Visuals`; the selected base name is stored
//! in config (`palette`), Light/Dark follows the OS color scheme.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Palette {
    System,
    Qlementine,
    Adwaita,
    NewAdwaita,
    Classic,
}

impl Palette {
    pub fn all_names() -> &'static [&'static str] {
        &["System", "Qlementine", "Adwaita", "New Adwaita", "Classic"]
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "Qlementine" => Self::Qlementine,
            "Adwaita" => Self::Adwaita,
            "New Adwaita" => Self::NewAdwaita,
            "Classic" => Self::Classic,
            _ => Self::System,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Qlementine => "Qlementine",
            Self::Adwaita => "Adwaita",
            Self::NewAdwaita => "New Adwaita",
            Self::Classic => "Classic",
        }
    }
}

/// Color-scheme branch.
/// Stored in config as `colorScheme` ("System"/"Dark"/"Light").
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ColorScheme {
    #[default]
    System,
    Dark,
    Light,
}

impl ColorScheme {
    #[allow(dead_code)]
    pub fn all_names() -> &'static [&'static str] {
        &["System", "Dark", "Light"]
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "Dark" => Self::Dark,
            "Light" => Self::Light,
            _ => Self::System,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }

    /// Resolve to concrete dark/light. `System` follows the OS theme
    /// (via the `dark-light` crate).
    pub fn is_dark(self) -> bool {
        match self {
            Self::Dark => true,
            Self::Light => false,
            Self::System => !matches!(dark_light::detect(), dark_light::Mode::Light),
        }
    }
}

/// UI density styles.
/// Stored in config as `style` (lowercased).
pub mod style {
    pub const DEFAULT: &str = "default";
    pub const COMPACT: &str = "compact";
    pub const SPACIOUS: &str = "spacious";

    pub fn all_names() -> &'static [&'static str] {
        &[DEFAULT, COMPACT, SPACIOUS]
    }

    pub fn normalize(name: &str) -> &'static str {
        match name.to_lowercase().as_str() {
            "compact" => COMPACT,
            "spacious" => SPACIOUS,
            _ => DEFAULT,
        }
    }

    /// Applies spacing tweaks for the given style name.
    pub fn apply(ctx: &egui::Context, name: &str) {
        match normalize(name) {
            COMPACT => ctx.all_styles_mut(|style| {
                style.spacing.button_padding = egui::vec2(4.0, 2.0);
                style.spacing.item_spacing = egui::vec2(4.0, 2.0);
                style.spacing.menu_margin = egui::Margin::same(2);
            }),
            SPACIOUS => ctx.all_styles_mut(|style| {
                style.spacing.button_padding = egui::vec2(10.0, 6.0);
                style.spacing.item_spacing = egui::vec2(10.0, 6.0);
                style.spacing.menu_margin = egui::Margin::same(8);
            }),
            _ => {}
        }
    }
}

/// Applies the palette; `dark` selects the Light/Dark color-scheme branch.
pub fn apply(ctx: &egui::Context, palette: Palette, dark: bool) {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    match (palette, dark) {
        (Palette::System, _) => {}
        (Palette::Qlementine, true) => {
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(48, 52, 66);
            visuals.window_fill = egui::Color32::from_rgb(40, 43, 51);
            visuals.selection.bg_fill = egui::Color32::from_rgb(80, 134, 255);
        }
        (Palette::Qlementine, false) => {
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(231, 231, 231);
            visuals.window_fill = egui::Color32::from_rgb(243, 243, 243);
            visuals.selection.bg_fill = egui::Color32::from_rgb(24, 144, 255);
        }
        (Palette::Adwaita, true) => {
            visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(48, 48, 48);
            visuals.selection.bg_fill = egui::Color32::from_rgb(53, 133, 229);
        }
        (Palette::Adwaita, false) => {
            visuals.window_fill = egui::Color32::from_rgb(249, 249, 247);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(255, 255, 255);
            visuals.selection.bg_fill = egui::Color32::from_rgb(53, 133, 229);
        }
        (Palette::NewAdwaita, true) => {
            visuals.window_fill = egui::Color32::from_rgb(35, 34, 39);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(47, 46, 51);
            visuals.selection.bg_fill = egui::Color32::from_rgb(53, 133, 229);
        }
        (Palette::NewAdwaita, false) => {
            visuals.window_fill = egui::Color32::from_rgb(249, 249, 247);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(255, 255, 255);
            visuals.selection.bg_fill = egui::Color32::from_rgb(53, 133, 229);
        }
        (Palette::Classic, true) => {
            visuals.window_fill = egui::Color32::from_rgb(27, 28, 30);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(32, 35, 40);
            visuals.selection.bg_fill = egui::Color32::from_rgb(194, 231, 255);
        }
        (Palette::Classic, false) => {
            visuals.window_fill = egui::Color32::from_rgb(245, 241, 235);
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(235, 227, 215);
            visuals.selection.bg_fill = egui::Color32::from_rgb(87, 86, 86);
        }
    }
    ctx.set_visuals(visuals);
}
