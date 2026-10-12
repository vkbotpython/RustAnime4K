//! Color themes (port of `palettes.py`).
//!
//! The 9 Qt palettes become `egui::Visuals`; the selected base name is stored
//! in config (`palette`), Light/Dark follows the OS color scheme like the
//! Python version does via `styleHints().colorScheme()`.

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

/// Applies the palette; `dark` mirrors Qt's Light/Dark color-scheme branch.
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
