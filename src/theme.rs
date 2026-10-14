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
            // Default must reset whatever compact/spacious set before.
            // Previously this branch was a no-op, so switching back to
            // "default" kept the previous style's spacing.
            _ => ctx.all_styles_mut(|style| {
                let defaults = egui::Style::default();
                style.spacing.button_padding = defaults.spacing.button_padding;
                style.spacing.item_spacing = defaults.spacing.item_spacing;
                style.spacing.menu_margin = defaults.spacing.menu_margin;
            }),
        }
    }
}

/// Card surface fill derived from the palette's `window_fill`: shifted a
/// step away from the window background so cards read as separate surfaces
/// in both dark and light themes. (Do NOT build it from `faint_bg_color` —
/// that is a near-black overlay in both themes, giving black cards on a
/// white window.)
pub fn card_fill_for(window: egui::Color32) -> egui::Color32 {
    const DELTA: u8 = 18;
    let luma = 0.299 * f32::from(window.r())
        + 0.587 * f32::from(window.g())
        + 0.114 * f32::from(window.b());
    let shift = |c: u8| {
        if luma < 128.0 {
            c.saturating_add(DELTA)
        } else {
            c.saturating_sub(DELTA)
        }
    };
    egui::Color32::from_rgb(shift(window.r()), shift(window.g()), shift(window.b()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_fill_offsets_away_from_window() {
        // Dark window -> lighter card, light window -> darker card.
        let dark = card_fill_for(egui::Color32::from_gray(27));
        assert_eq!(dark, egui::Color32::from_gray(45));
        let light = card_fill_for(egui::Color32::from_gray(248));
        assert_eq!(light, egui::Color32::from_gray(230));
        // Tinted palettes shift per channel without overflow.
        let cream = card_fill_for(egui::Color32::from_rgb(245, 241, 235));
        assert_eq!(cream, egui::Color32::from_rgb(227, 223, 217));
        let near_black = card_fill_for(egui::Color32::from_rgb(0, 5, 250));
        assert_eq!(near_black, egui::Color32::from_rgb(18, 23, 255));
    }

    #[test]
    fn default_style_resets_compact_and_spacious_spacing() {
        let ctx = egui::Context::default();
        let expected = egui::Style::default();
        let spacing = || {
            ctx.all_styles_mut(|_| {});
            // Read back via style mutation probe: capture current values.
            let mut current = None;
            ctx.all_styles_mut(|s| {
                current = Some((
                    s.spacing.button_padding,
                    s.spacing.item_spacing,
                    s.spacing.menu_margin,
                ));
            });
            current.expect("style must exist")
        };

        style::apply(&ctx, style::COMPACT);
        assert_ne!(spacing().0, expected.spacing.button_padding);
        style::apply(&ctx, style::DEFAULT);
        assert_eq!(spacing(), {
            (
                expected.spacing.button_padding,
                expected.spacing.item_spacing,
                expected.spacing.menu_margin,
            )
        });

        style::apply(&ctx, style::SPACIOUS);
        assert_ne!(spacing().0, expected.spacing.button_padding);
        style::apply(&ctx, style::DEFAULT);
        assert_eq!(spacing().0, expected.spacing.button_padding);
    }
}
