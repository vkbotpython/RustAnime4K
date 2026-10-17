//! Color themes.
//!
//! The 9 palettes become `egui::Visuals`; the selected base name is stored
//! in config (`palette`), Light/Dark follows the OS color scheme. A window
//! backdrop material (Mica/Acrylic, see `crate::mica`) additionally makes
//! the surface fills translucent so the material shows through.

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
            Self::System => !matches!(dark_light::detect(), Ok(dark_light::Mode::Light)),
        }
    }
}

/// Window backdrop material (Windows 11). `Off` is a plain opaque window;
/// every other variant makes the window shell translucent so the DWM
/// material drawn behind it (`crate::mica`) shows through. Persisted in
/// config as `backdrop`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Backdrop {
    #[default]
    Off,
    Mica,
    MicaAlt,
    Acrylic,
}

impl Backdrop {
    /// Selectable names (`Off` first). `all_names` is only used by the
    /// Windows-only settings row; other platforms hide the setting.
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn all_names() -> &'static [&'static str] {
        &["Off", "Mica", "Mica Alt", "Acrylic"]
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "Mica" => Self::Mica,
            "Mica Alt" => Self::MicaAlt,
            "Acrylic" => Self::Acrylic,
            _ => Self::Off,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Mica => "Mica",
            Self::MicaAlt => "Mica Alt",
            Self::Acrylic => "Acrylic",
        }
    }

    /// True for every material, i.e. while the window is translucent.
    pub fn enabled(self) -> bool {
        self != Self::Off
    }

    /// Material actually applicable on this platform: Mica/Acrylic are
    /// Windows 11 materials and are ported via config (a machine sync or a
    /// copied `config.json`) — elsewhere the window stays opaque.
    pub fn effective(self) -> Self {
        if cfg!(windows) { self } else { Self::Off }
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
/// white window.) Under a backdrop material `window_fill` arrives
/// premultiplied (possibly translucent): unmultiply first, or the cards
/// come out darker than the window.
pub fn card_fill_for(window: egui::Color32) -> egui::Color32 {
    const DELTA: u8 = 18;
    let [r, g, b, _] = window.to_srgba_unmultiplied();
    let luma = 0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b);
    let shift = |c: u8| {
        if luma < 128.0 {
            c.saturating_add(DELTA)
        } else {
            c.saturating_sub(DELTA)
        }
    };
    egui::Color32::from_rgb(shift(r), shift(g), shift(b))
}

/// Surface opacity under a backdrop material: panels are the most
/// transparent (the material should be clearly visible through them),
/// while popups (dialogs, menus) stay close to opaque so text on them
/// keeps its contrast.
const PANEL_OPACITY: f32 = 0.78;
const POPUP_OPACITY: f32 = 0.91;

/// Alpha-scales the surface fills so the DWM material behind the window
/// (see `crate::mica`) shines through. Only backgrounds are touched —
/// text, controls and strokes keep their colors and stay readable.
fn make_translucent(visuals: &mut egui::Visuals) {
    visuals.panel_fill = visuals.panel_fill.gamma_multiply(PANEL_OPACITY);
    visuals.window_fill = visuals.window_fill.gamma_multiply(POPUP_OPACITY);
    visuals.extreme_bg_color = visuals.extreme_bg_color.gamma_multiply(POPUP_OPACITY);
    visuals.code_bg_color = visuals.code_bg_color.gamma_multiply(POPUP_OPACITY);
}

/// Applies the palette; `dark` selects the Light/Dark color-scheme branch,
/// `backdrop` makes the window shell translucent (Mica/Acrylic).
pub fn apply(ctx: &egui::Context, palette: Palette, dark: bool, backdrop: Backdrop) {
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
    if backdrop.enabled() {
        make_translucent(&mut visuals);
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
    fn card_fill_ignores_alpha_of_translucent_window() {
        // Backdrop materials make `window_fill` premultiplied/translucent;
        // the card color must not change because of that (±1 per channel
        // for the premultiply/unmultiply round trip).
        let opaque = card_fill_for(egui::Color32::from_rgb(40, 43, 51)).to_srgba_unmultiplied();
        let translucent = card_fill_for(egui::Color32::from_rgba_unmultiplied(40, 43, 51, 199))
            .to_srgba_unmultiplied();
        for (o, t) in opaque.iter().zip(translucent.iter()) {
            assert!(o.abs_diff(*t) <= 1, "{opaque:?} vs {translucent:?}");
        }
    }

    #[test]
    fn backdrop_names_roundtrip() {
        for &name in Backdrop::all_names() {
            assert_eq!(Backdrop::from_name(name).name(), name);
        }
        assert_eq!(Backdrop::from_name("???"), Backdrop::Off);
        assert!(!Backdrop::Off.enabled());
        for material in [Backdrop::Mica, Backdrop::MicaAlt, Backdrop::Acrylic] {
            assert!(material.enabled());
        }
    }

    #[test]
    fn backdrop_makes_surfaces_translucent_without_recoloring() {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Dark);
        apply(&ctx, Palette::Adwaita, true, Backdrop::Off);
        let (opaque_panel, opaque_window) = {
            let style = ctx.global_style();
            assert_eq!(style.visuals.panel_fill.a(), 255);
            assert_eq!(style.visuals.window_fill.a(), 255);
            (style.visuals.panel_fill, style.visuals.window_fill)
        };

        apply(&ctx, Palette::Adwaita, true, Backdrop::Mica);
        let style = ctx.global_style();
        // Both shells are translucent, panels clearly more so, and only the
        // alpha changed — the surface colors stay the same.
        assert!(style.visuals.panel_fill.a() < style.visuals.window_fill.a());
        assert!(style.visuals.window_fill.a() < 255);
        assert_eq!(
            style.visuals.panel_fill.to_srgba_unmultiplied()[..3],
            opaque_panel.to_srgba_unmultiplied()[..3]
        );
        assert_eq!(
            style.visuals.window_fill.to_srgba_unmultiplied()[..3],
            opaque_window.to_srgba_unmultiplied()[..3]
        );
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
