//! Color themes.
//!
//! Controls always draw in the Windows 11 (WinUI 3 / Fluent) language: the
//! colors below are the XAML theme resources from `microsoft-ui-xaml`
//! (`CommonStyles/*_themeresources.xaml`), so every value can be re-checked
//! against the original. A palette (config `palette`) only picks the base
//! surface (window/panels) and the accent color; Light/Dark follows the OS
//! color scheme (`colorScheme`). A window backdrop material (Mica/Acrylic,
//! see `crate::mica`) additionally makes the surface fills translucent so
//! the material shows through.
//!
//! egui couples a few colors that WinUI keeps apart — a checkbox box and a
//! slider rail share `widgets.inactive.bg_fill`, a button label and a
//! check-mark share `fg_stroke` — so [`checkbox`] and [`slider`] re-point
//! those fields for the duration of a single widget.

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

    /// WinUI metrics shared by every density: the standard control height
    /// (`ButtonMinHeight` 32 ends up near this once the text is centered),
    /// the 4px slider track and the 16px checkbox box (`CheckBoxSize` 20,
    /// shrunk for egui's denser rows). `indent` keeps the egui invariant
    /// `button_padding.x + icon_width + icon_spacing`.
    pub const CONTROL_HEIGHT: f32 = 24.0;
    /// Slider row height. egui derives the handle radius from the row
    /// (`height / 2.5`), so sliders get their own compact row: a transport
    /// or volume thumb should not be as tall as a button (~13px instead of
    /// the 19px ball a control-height row would produce).
    pub const SLIDER_HEIGHT: f32 = 16.0;
    pub const SLIDER_RAIL_HEIGHT: f32 = 4.0;
    pub const ICON_WIDTH: f32 = 16.0;
    pub const ICON_WIDTH_INNER: f32 = 10.0;
    pub const ICON_SPACING: f32 = 6.0;
    pub const INDENT: f32 = 32.0;

    /// Padding of a `default` button: WinUI `ButtonPadding` is
    /// `11,5,11,6`, rounded to what egui's `control_height` fits.
    const DEFAULT_BUTTON_PADDING: (f32, f32) = (10.0, 4.0);
    const DEFAULT_ITEM_SPACING: (f32, f32) = (8.0, 4.0);
    const DEFAULT_MENU_MARGIN: i8 = 6;
    /// Window frame margin: `SidePanel` and dialogs sit a bit tighter than
    /// egui's default 6.
    pub const WINDOW_MARGIN: i8 = 8;

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

    /// WinUI scroll bars through egui's "floating" preset: a thin handle
    /// that stays visible next to the content, brightens while the scroll
    /// area is hovered and widens into a grabbable bar under the pointer.
    pub fn scroll_style() -> egui::style::ScrollStyle {
        let mut scroll = egui::style::ScrollStyle::floating();
        scroll.bar_width = 6.0;
        scroll.floating_width = 3.0;
        scroll.floating_allocated_width = 3.0;
        scroll.handle_min_length = 24.0;
        scroll.dormant_handle_opacity = 0.4;
        scroll
    }

    /// Applies spacing tweaks for the given style name. The Fluent base
    /// metrics are (re)applied for every density, so switching between
    /// compact/default/spacious can never drift.
    pub fn apply(ctx: &egui::Context, name: &str) {
        let (button_padding, item_spacing, menu_margin) = match normalize(name) {
            COMPACT => (
                egui::vec2(4.0, 2.0),
                egui::vec2(4.0, 2.0),
                egui::Margin::same(2),
            ),
            SPACIOUS => (
                egui::vec2(10.0, 6.0),
                egui::vec2(10.0, 6.0),
                egui::Margin::same(8),
            ),
            _ => (
                egui::vec2(DEFAULT_BUTTON_PADDING.0, DEFAULT_BUTTON_PADDING.1),
                egui::vec2(DEFAULT_ITEM_SPACING.0, DEFAULT_ITEM_SPACING.1),
                egui::Margin::same(DEFAULT_MENU_MARGIN),
            ),
        };
        ctx.all_styles_mut(|style| {
            let spacing = &mut style.spacing;
            spacing.button_padding = button_padding;
            spacing.item_spacing = item_spacing;
            spacing.menu_margin = menu_margin;
            spacing.window_margin = egui::Margin::same(WINDOW_MARGIN);
            spacing.interact_size.y = CONTROL_HEIGHT;
            spacing.slider_rail_height = SLIDER_RAIL_HEIGHT;
            spacing.icon_width = ICON_WIDTH;
            spacing.icon_width_inner = ICON_WIDTH_INNER;
            spacing.icon_spacing = ICON_SPACING;
            spacing.indent = INDENT;
            spacing.scroll = scroll_style();
        });
    }
}

/// WinUI 3 (XAML) resource colors for one color scheme. The names are the
/// original resource keys (`Common_themeresources_any.xaml`); unsuffixed
/// entries are the `...Default` variants.
struct Fluent {
    /// `SolidBackgroundFillColorBase`: window/dialog background.
    surface: egui::Color32,
    /// `ControlFillColorDefault`: rest fill of buttons and input fields.
    control_fill: egui::Color32,
    /// `ControlFillColorSecondary` (hover) / `Tertiary` (pressed).
    control_fill_hover: egui::Color32,
    control_fill_pressed: egui::Color32,
    /// `ControlStrongFillColorDefault`: slider rail and thumb at rest.
    control_strong: egui::Color32,
    /// `ControlStrongStrokeColorDefault` (`Disabled`): unchecked checkbox.
    control_strong_stroke: egui::Color32,
    control_strong_stroke_disabled: egui::Color32,
    /// `ControlAltFillColorSecondary/Tertiary/Quarternary`:
    /// checkbox box at rest/hover/pressed.
    control_alt_fill: [egui::Color32; 3],
    /// Shared border of buttons, checkboxes and input fields: the average of
    /// `ControlElevationBorderBrush` (`ControlStrokeColorSecondary` at the
    /// top, `ControlStrokeColorDefault` at the bottom) — egui strokes a
    /// single color where WinUI paints a gradient.
    control_border: egui::Color32,
    /// `DividerStrokeColorDefault` (separators).
    divider: egui::Color32,
    /// `SurfaceStrokeColorDefault`: outline of windows and menus.
    surface_stroke: egui::Color32,
    /// `CardStrokeColorDefault`.
    card_stroke: egui::Color32,
    /// Alpha of `CardBackgroundFillColorDefault`, layered over the window
    /// surface to make cards sit one step lighter (see `card_fill_for`).
    card_layer: u8,
    /// `TextFillColorPrimary` / `Tertiary` (regular and faint text).
    text: egui::Color32,
    text_tertiary: egui::Color32,
    /// `AccentFillColorDefault` and `AccentTextFillColorPrimary`.
    accent: egui::Color32,
    accent_text: egui::Color32,
    /// Drop shadow of windows and popups (Fluent dialogs float).
    shadow: egui::Color32,
    /// `SystemFillColorCaution` / `Critical` (warning and error text).
    warn: egui::Color32,
    error: egui::Color32,
}

/// `Color32::from_black_alpha` / `from_white_alpha` as short locals: the
/// WinUI tables below read much better as `black(0x29)`.
fn black(alpha: u8) -> egui::Color32 {
    egui::Color32::from_black_alpha(alpha)
}

fn white(alpha: u8) -> egui::Color32 {
    egui::Color32::from_white_alpha(alpha)
}

impl Fluent {
    fn dark() -> Self {
        Self {
            surface: egui::Color32::from_rgb(0x20, 0x20, 0x20),
            control_fill: white(0x0F),
            control_fill_hover: white(0x15),
            control_fill_pressed: white(0x08),
            control_strong: white(0x8B),
            control_strong_stroke: white(0x8B),
            control_strong_stroke_disabled: white(0x28),
            control_alt_fill: [black(0x19), white(0x0B), white(0x12)],
            control_border: white(0x33),
            divider: white(0x15),
            surface_stroke: egui::Color32::from_rgba_unmultiplied(0x75, 0x75, 0x75, 0x66),
            card_stroke: black(0x19),
            card_layer: 0x0D,
            text: egui::Color32::WHITE,
            text_tertiary: white(0x87),
            accent: egui::Color32::from_rgb(0x60, 0xCD, 0xFF),
            accent_text: egui::Color32::from_rgb(0x99, 0xEB, 0xFF),
            shadow: black(0x59),
            warn: egui::Color32::from_rgb(0xFC, 0xE1, 0x00),
            error: egui::Color32::from_rgb(0xFF, 0x99, 0xA4),
        }
    }

    fn light() -> Self {
        Self {
            surface: egui::Color32::from_rgb(0xF3, 0xF3, 0xF3),
            control_fill: white(0xB3),
            control_fill_hover: egui::Color32::from_rgba_unmultiplied(0xF9, 0xF9, 0xF9, 0x80),
            control_fill_pressed: egui::Color32::from_rgba_unmultiplied(0xF9, 0xF9, 0xF9, 0x4D),
            control_strong: black(0x72),
            control_strong_stroke: black(0x72),
            control_strong_stroke_disabled: black(0x37),
            control_alt_fill: [black(0x06), black(0x0F), black(0x18)],
            control_border: black(0x3D),
            divider: black(0x0F),
            surface_stroke: egui::Color32::from_rgba_unmultiplied(0x75, 0x75, 0x75, 0x66),
            card_stroke: black(0x0F),
            card_layer: 0xB3,
            text: black(0xE4),
            text_tertiary: black(0x72),
            accent: egui::Color32::from_rgb(0x00, 0x78, 0xD4),
            accent_text: egui::Color32::from_rgb(0x00, 0x5F, 0xB8),
            shadow: black(0x24),
            warn: egui::Color32::from_rgb(0x9D, 0x5D, 0x00),
            error: egui::Color32::from_rgb(0xC4, 0x2B, 0x1C),
        }
    }

    fn for_scheme(dark: bool) -> Self {
        if dark { Self::dark() } else { Self::light() }
    }
}

/// Base visuals of the Fluent look: every control color comes from the
/// resources above; the palette-dependent surface and accent are filled in
/// by [`apply`].
fn fluent_visuals(dark: bool) -> egui::Visuals {
    use egui::style::{HandleShape, WidgetVisuals};
    use egui::{Stroke, style::TextCursorStyle};

    let c = Fluent::for_scheme(dark);
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    // ControlCornerRadius.
    let corner = egui::CornerRadius::same(4);
    let border = Stroke::new(1.0, c.control_border);
    // `fg_stroke.width` doubles as the check-mark / slider-thumb stroke.
    let text = Stroke::new(1.5, c.text);

    // `noninteractive.bg_fill` — the surface of `Frame::group` and custom
    // frames; `bg_stroke` — separators and group outlines.
    visuals.widgets.noninteractive = WidgetVisuals {
        bg_fill: c.surface,
        weak_bg_fill: c.surface,
        bg_stroke: Stroke::new(1.0, c.divider),
        fg_stroke: Stroke::new(1.0, c.text),
        corner_radius: corner,
        expansion: 0.0,
    };
    // Buttons (weak_bg_fill), checkboxes and the slider rail (bg_fill).
    visuals.widgets.inactive = WidgetVisuals {
        bg_fill: c.control_strong,
        weak_bg_fill: c.control_fill,
        bg_stroke: border,
        fg_stroke: text,
        corner_radius: corner,
        expansion: 0.0,
    };
    visuals.widgets.hovered = WidgetVisuals {
        bg_fill: c.control_fill_hover,
        weak_bg_fill: c.control_fill_hover,
        bg_stroke: border,
        fg_stroke: text,
        corner_radius: corner,
        expansion: 0.0,
    };
    // `bg_fill` here is the accent (checked boxes, dragged slider thumb);
    // `fg_stroke` must stay the regular text color — `Visuals::strong_text_color`
    // reads it, so bold labels would otherwise dim or take the on-accent color.
    visuals.widgets.active = WidgetVisuals {
        bg_fill: c.accent,
        weak_bg_fill: c.control_fill_pressed,
        bg_stroke: border,
        fg_stroke: text,
        corner_radius: corner,
        expansion: 0.0,
    };
    // An open menu / combo button.
    visuals.widgets.open = WidgetVisuals {
        bg_fill: c.control_fill_hover,
        weak_bg_fill: c.control_fill_hover,
        bg_stroke: border,
        fg_stroke: text,
        corner_radius: corner,
        expansion: 0.0,
    };

    visuals.weak_text_color = Some(c.text_tertiary);
    visuals.hyperlink_color = c.accent_text;
    visuals.faint_bg_color = c.control_fill;
    // Text fields sit one step off the surface (`ControlFillColorDefault`
    // in WinUI); the scroll-bar track reuses this color faintly.
    visuals.extreme_bg_color = c.control_fill;
    visuals.code_bg_color = c.control_fill_hover;
    visuals.warn_fg_color = c.warn;
    visuals.error_fg_color = c.error;

    // OverlayCornerRadius: windows and menus.
    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0, c.surface_stroke);
    visuals.window_shadow = egui::Shadow {
        offset: [0, 8],
        blur: 20,
        spread: 0,
        color: c.shadow,
    };
    visuals.popup_shadow = egui::Shadow {
        offset: [0, 6],
        blur: 14,
        spread: 0,
        color: c.shadow,
    };
    // Windows are not outlined brighter just for being on top.
    visuals.window_highlight_topmost = false;
    visuals.window_fill = c.surface;
    visuals.panel_fill = c.surface;

    visuals.text_cursor = TextCursorStyle {
        stroke: Stroke::new(2.0, c.text),
        ..Default::default()
    };
    // Accent sliders: the track up to the handle is filled with the accent.
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = HandleShape::Circle;
    visuals.button_frame = true;
    visuals.striped = false;

    visuals
}

/// Accent-secondary/-tertiary fills: WinUI derives them from the accent by
/// alpha (`AccentFillColorSecondary` is ~90%, `Tertiary` ~80%).
fn accent_shade(accent: egui::Color32, factor: f32) -> egui::Color32 {
    accent.gamma_multiply(factor)
}

/// Text color that reads on top of the accent (`TextOnAccentFillColorPrimary`):
/// black on light accents, white on dark ones. The threshold sits between
/// the light system accent (`#60CDFF`, needs black) and the darker palette
/// accents like Adwaita blue (need white).
fn on_accent(accent: egui::Color32) -> egui::Color32 {
    let [r, g, b, _] = accent.to_srgba_unmultiplied();
    let luma = 0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b);
    if luma > 0.65 * 255.0 {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    }
}

/// Card surface fill (WinUI `CardBackgroundFillColorDefault`): a white layer
/// over the window surface, so cards sit one step lighter than the window in
/// both themes. Under a backdrop material `window_fill` arrives
/// premultiplied (possibly translucent): unmultiply first, or the cards come
/// out darker than the window. The window's alpha is preserved so cards stay
/// as translucent as the shell around them.
pub fn card_fill_for(window: egui::Color32, dark: bool) -> egui::Color32 {
    let layer = Fluent::for_scheme(dark).card_layer;
    let mix = f32::from(layer) / 255.0;
    let [r, g, b, a] = window.to_srgba_unmultiplied();
    let blend = |c: u8| (f32::from(c) * (1.0 - mix) + 255.0 * mix).round() as u8;
    egui::Color32::from_rgba_unmultiplied(blend(r), blend(g), blend(b), a)
}

/// Outline of a card (`CardStrokeColorDefault`); `noninteractive.bg_stroke`
/// is reserved for separators and would be too faint.
pub fn card_stroke(ui: &egui::Ui) -> egui::Color32 {
    Fluent::for_scheme(ui.visuals().dark_mode).card_stroke
}

/// Corner radius of cards: WinUI rounds them like ordinary controls
/// (`ControlCornerRadius`), not like windows and menus.
pub const CARD_CORNER_RADIUS: u8 = 4;

/// Accent fill of the current palette (selected rows, checked boxes).
pub fn accent(ui: &egui::Ui) -> egui::Color32 {
    ui.visuals().selection.bg_fill
}

/// Text color that reads on top of [`accent`].
pub fn on_accent_of(ui: &egui::Ui) -> egui::Color32 {
    ui.visuals().selection.stroke.color
}

/// WinUI checkbox. egui paints the box from `widgets.*.bg_fill` — the same
/// field the slider rail reads — and the check-mark from `fg_stroke` — the
/// same field the label reads. Both are re-pointed here, using the checked
/// value of the previous pass (styles are one pass behind anyway):
/// unchecked boxes get the `ControlAltFillColor*` wash with the strong
/// `ControlStrongStrokeColorDefault` border, checked ones the accent fill
/// with an on-accent check-mark while the label keeps the regular text color.
pub fn checkbox(ui: &mut egui::Ui, checked: &mut bool, label: &str) -> egui::Response {
    let c = Fluent::for_scheme(ui.visuals().dark_mode);
    let accent = accent(ui);
    let glyph = on_accent_of(ui);
    let label_color = ui.visuals().text_color();
    let on = *checked;

    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        if on {
            widgets.inactive.bg_fill = accent;
            widgets.inactive.bg_stroke = egui::Stroke::new(1.0, accent);
            widgets.inactive.fg_stroke = egui::Stroke::new(1.5, glyph);
            widgets.hovered.bg_fill = accent_shade(accent, 0.9);
            widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent_shade(accent, 0.9));
            widgets.hovered.fg_stroke = egui::Stroke::new(1.5, glyph);
            widgets.active.bg_fill = accent_shade(accent, 0.8);
            widgets.active.bg_stroke = egui::Stroke::new(1.0, accent_shade(accent, 0.8));
            // TextOnAccentFillColorSecondary: the glyph fades while pressed.
            widgets.active.fg_stroke = egui::Stroke::new(1.5, glyph.gamma_multiply(0.8));
        } else {
            widgets.inactive.bg_fill = c.control_alt_fill[0];
            widgets.inactive.bg_stroke = egui::Stroke::new(1.0, c.control_strong_stroke);
            widgets.hovered.bg_fill = c.control_alt_fill[1];
            widgets.hovered.bg_stroke = egui::Stroke::new(1.0, c.control_strong_stroke);
            widgets.active.bg_fill = c.control_alt_fill[2];
            widgets.active.bg_stroke = egui::Stroke::new(1.0, c.control_strong_stroke_disabled);
        }
        // The label is pinned to the regular text color: the state visuals
        // above may carry the on-accent check-mark color in `fg_stroke`.
        ui.checkbox(checked, egui::RichText::new(label).color(label_color))
    })
    .inner
}

/// WinUI slider: the rail and the resting thumb are
/// `ControlStrongFillColorDefault` (the global `inactive.bg_fill`), the
/// fill up to the handle is the accent, and a hovered thumb brightens to
/// the accent (`AccentFillColorSecondary`). egui paints the hovered thumb
/// from `widgets.hovered.bg_fill` — shared with the hovered checkbox box —
/// so the accent is only applied around the slider itself.
pub fn slider<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let accent = accent(ui);
    ui.scope(|ui| {
        // egui sizes the handle from the row height (radius = height / 2.5).
        // A compact row keeps the thumb from covering the position like a
        // button-sized ball would on a transport bar.
        ui.spacing_mut().interact_size.y = style::SLIDER_HEIGHT;
        ui.visuals_mut().widgets.hovered.bg_fill = accent_shade(accent, 0.9);
        add(ui)
    })
    .inner
}

/// WinUI focus cue of an input field: a 2px accent underline along the
/// bottom edge. egui outlines the focused field with `selection.stroke` — the
/// on-accent glyph color, which selected text needs but which is nearly
/// invisible against the field fill — so the underline carries the accent.
pub fn focus_underline(ui: &egui::Ui, response: &egui::Response) {
    if response.has_focus() {
        let rect = response.rect;
        let y = rect.bottom() - 1.0;
        let inset = f32::from(CARD_CORNER_RADIUS);
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + inset, y),
                egui::pos2(rect.right() - inset, y),
            ],
            egui::Stroke::new(2.0, accent(ui)),
        );
    }
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

/// Base surface (window, panels) and accent color of a palette; everything
/// else is Fluent (see `fluent_visuals`).
fn palette_colors(palette: Palette, dark: bool) -> (egui::Color32, egui::Color32, egui::Color32) {
    let rgb = egui::Color32::from_rgb;
    match (palette, dark) {
        // System: the WinUI defaults.
        (Palette::System, true) => (rgb(32, 32, 32), rgb(32, 32, 32), rgb(96, 205, 255)),
        (Palette::System, false) => (rgb(243, 243, 243), rgb(243, 243, 243), rgb(0, 120, 212)),
        (Palette::Qlementine, true) => (rgb(40, 43, 51), rgb(48, 52, 66), rgb(80, 134, 255)),
        (Palette::Qlementine, false) => (rgb(243, 243, 243), rgb(231, 231, 231), rgb(24, 144, 255)),
        (Palette::Adwaita, true) => (rgb(30, 30, 30), rgb(48, 48, 48), rgb(53, 133, 229)),
        (Palette::Adwaita, false) => (rgb(249, 249, 247), rgb(255, 255, 255), rgb(53, 133, 229)),
        (Palette::NewAdwaita, true) => (rgb(35, 34, 39), rgb(47, 46, 51), rgb(53, 133, 229)),
        (Palette::NewAdwaita, false) => (rgb(249, 249, 247), rgb(255, 255, 255), rgb(53, 133, 229)),
        (Palette::Classic, true) => (rgb(27, 28, 30), rgb(32, 35, 40), rgb(194, 231, 255)),
        (Palette::Classic, false) => (rgb(245, 241, 235), rgb(235, 227, 215), rgb(87, 86, 86)),
    }
}

/// Applies the theme; `dark` selects the Light/Dark color-scheme branch,
/// `backdrop` makes the window shell translucent (Mica/Acrylic).
pub fn apply(ctx: &egui::Context, palette: Palette, dark: bool, backdrop: Backdrop) {
    let mut visuals = fluent_visuals(dark);
    let (window, panel, accent) = palette_colors(palette, dark);
    visuals.window_fill = window;
    visuals.panel_fill = panel;
    visuals.widgets.noninteractive.bg_fill = panel;
    visuals.widgets.noninteractive.weak_bg_fill = panel;
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = egui::Stroke::new(1.0, on_accent(accent));
    // Checked checkboxes (see `checkbox`) and the dragged slider thumb.
    visuals.widgets.active.bg_fill = accent;
    if backdrop.enabled() {
        make_translucent(&mut visuals);
    }
    // The Fluent look is the app's only look: fill both theme slots so a
    // later OS/theme switch cannot resurface egui's default visuals.
    ctx.set_visuals_of(egui::Theme::Dark, visuals.clone());
    ctx.set_visuals_of(egui::Theme::Light, visuals);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_fill_is_a_light_layer_over_the_window() {
        // CardBackgroundFillColorDefault: a white layer over the window —
        // cards are lighter than the window in both themes.
        let dark = card_fill_for(egui::Color32::from_gray(32), true).to_srgba_unmultiplied();
        assert!(dark[0] > 32, "dark cards lighten: {dark:?}");
        let light = card_fill_for(egui::Color32::from_gray(243), false).to_srgba_unmultiplied();
        assert!(light[0] > 243, "light cards lighten: {light:?}");
        // Tinted palettes shift per channel without overflow.
        let cream =
            card_fill_for(egui::Color32::from_rgb(245, 241, 235), false).to_srgba_unmultiplied();
        assert!(cream[0] >= 245 && cream[2] >= 235, "{cream:?}");
        let near_white =
            card_fill_for(egui::Color32::from_rgb(252, 250, 248), false).to_srgba_unmultiplied();
        assert!(
            near_white.iter().take(3).all(|&c| c >= 250),
            "{near_white:?}"
        );
    }

    #[test]
    fn card_fill_ignores_alpha_of_translucent_window() {
        // Backdrop materials make `window_fill` premultiplied/translucent;
        // the card color must not change because of that (±1 per channel
        // for the premultiply/unmultiply round trip) and the window's alpha
        // must survive, or cards would be more opaque than the shell.
        let opaque =
            card_fill_for(egui::Color32::from_rgb(40, 43, 51), true).to_srgba_unmultiplied();
        let translucent =
            card_fill_for(egui::Color32::from_rgba_unmultiplied(40, 43, 51, 199), true)
                .to_srgba_unmultiplied();
        assert_eq!(translucent[3], 199);
        for (o, t) in opaque[..3].iter().zip(translucent[..3].iter()) {
            assert!(o.abs_diff(*t) <= 1, "{opaque:?} vs {translucent:?}");
        }
    }

    #[test]
    fn on_accent_reads_on_every_palette_accent() {
        // Light system accent -> black text, the darker accents -> white.
        assert_eq!(
            on_accent(egui::Color32::from_rgb(96, 205, 255)),
            egui::Color32::BLACK
        );
        assert_eq!(
            on_accent(egui::Color32::from_rgb(194, 231, 255)),
            egui::Color32::BLACK
        );
        assert_eq!(
            on_accent(egui::Color32::from_rgb(0, 120, 212)),
            egui::Color32::WHITE
        );
        assert_eq!(
            on_accent(egui::Color32::from_rgb(53, 133, 229)),
            egui::Color32::WHITE
        );
        assert_eq!(
            on_accent(egui::Color32::from_rgb(87, 86, 86)),
            egui::Color32::WHITE
        );
    }

    #[test]
    fn checked_checkbox_glyph_follows_the_palette_accent() {
        // Adwaita in the dark scheme: the accent is a dark blue, so WinUI
        // wants a white check-mark even though the dark scheme's default
        // TextOnAccent is black. The glyph must follow the real accent.
        let themed = egui::Context::default();
        themed.set_theme(egui::ThemePreference::Dark);
        apply(&themed, Palette::Adwaita, true, Backdrop::Off);
        let visuals = themed.global_style().visuals.clone();

        let ctx = egui::Context::default();
        ctx.set_fonts(egui::FontDefinitions::empty());
        let mut checked = true;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            *ui.visuals_mut() = visuals.clone();
            checkbox(ui, &mut checked, "test");
        });

        let accent = egui::Color32::from_rgb(53, 133, 229);
        let white_check_mark = egui::epaint::PathStroke::new(1.5, egui::Color32::WHITE);
        let (mut accent_box, mut white_glyph) = (false, false);
        for clipped in &output.shapes {
            match &clipped.shape {
                egui::Shape::Rect(rect) if rect.fill == accent => accent_box = true,
                egui::Shape::Path(path) if path.stroke == white_check_mark => white_glyph = true,
                _ => {}
            }
        }
        output.drop_without_applying_deltas();
        assert!(accent_box, "a checked box is filled with the accent");
        assert!(white_glyph, "the check-mark stays readable on the accent");
    }

    #[test]
    fn apply_writes_both_theme_slots() {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Dark);
        apply(&ctx, Palette::System, true, Backdrop::Off);
        ctx.set_theme(egui::ThemePreference::Light);
        // Switching the active theme slot must not resurface egui's defaults.
        let v = ctx.global_style().visuals.clone();
        assert_eq!(v.panel_fill, egui::Color32::from_rgb(32, 32, 32));
        assert_eq!(
            v.widgets.inactive.bg_fill,
            egui::Color32::from_white_alpha(0x8B)
        );
    }

    #[test]
    fn slider_uses_a_compact_handle_row() {
        egui::__run_test_ui(|ui| {
            slider(ui, |ui| {
                assert_eq!(ui.spacing().interact_size.y, style::SLIDER_HEIGHT);
            });
        });
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
    fn system_palette_uses_winui_resources() {
        use egui::style::HandleShape;
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Dark);
        apply(&ctx, Palette::System, true, Backdrop::Off);
        let v = ctx.global_style().visuals.clone();

        // Surfaces and accent.
        assert_eq!(v.panel_fill, egui::Color32::from_rgb(32, 32, 32));
        assert_eq!(v.window_fill, egui::Color32::from_rgb(32, 32, 32));
        assert_eq!(v.selection.bg_fill, egui::Color32::from_rgb(96, 205, 255));
        // Black glyphs on the light-blue accent.
        assert_eq!(v.selection.stroke.color, egui::Color32::BLACK);
        // Geometry: OverlayCornerRadius for windows, ControlCornerRadius for
        // controls, circle slider thumbs, accent trailing fill.
        assert_eq!(v.window_corner_radius, egui::CornerRadius::same(8));
        assert_eq!(v.menu_corner_radius, egui::CornerRadius::same(8));
        assert_eq!(
            v.widgets.inactive.corner_radius,
            egui::CornerRadius::same(4)
        );
        assert_eq!(v.handle_shape, HandleShape::Circle);
        assert!(v.slider_trailing_fill);
        assert!(!v.window_highlight_topmost);
        // Control fills: buttons (weak) vs slider rail (strong).
        assert_eq!(
            v.widgets.inactive.weak_bg_fill,
            egui::Color32::from_white_alpha(0x0F)
        );
        assert_eq!(
            v.widgets.inactive.bg_fill,
            egui::Color32::from_white_alpha(0x8B)
        );
        assert_eq!(
            v.widgets.hovered.weak_bg_fill,
            egui::Color32::from_white_alpha(0x15)
        );
        assert_eq!(
            v.widgets.active.bg_fill,
            egui::Color32::from_rgb(96, 205, 255)
        );
    }

    #[test]
    fn palette_keeps_fluent_controls_but_changes_surface_and_accent() {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Dark);
        apply(&ctx, Palette::Qlementine, true, Backdrop::Off);
        let v = ctx.global_style().visuals.clone();
        assert_eq!(v.window_fill, egui::Color32::from_rgb(40, 43, 51));
        assert_eq!(v.panel_fill, egui::Color32::from_rgb(48, 52, 66));
        assert_eq!(v.selection.bg_fill, egui::Color32::from_rgb(80, 134, 255));
        // White glyphs: the Qlementine accent is dark enough.
        assert_eq!(v.selection.stroke.color, egui::Color32::WHITE);
        // Controls are untouched by the palette.
        assert_eq!(
            v.widgets.inactive.bg_fill,
            egui::Color32::from_white_alpha(0x8B)
        );
        assert_eq!(
            v.widgets.active.bg_fill,
            egui::Color32::from_rgb(80, 134, 255)
        );
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
        // Both shells are translucent, panels clearly more so, and the
        // surface colors stay the same — up to the ±1 of the premultiplied
        // storage round trip.
        assert!(style.visuals.panel_fill.a() < style.visuals.window_fill.a());
        assert!(style.visuals.window_fill.a() < 255);
        let same_rgb = |a: egui::Color32, b: egui::Color32| {
            a.to_srgba_unmultiplied()[..3]
                .iter()
                .zip(b.to_srgba_unmultiplied()[..3].iter())
                .all(|(x, y)| x.abs_diff(*y) <= 1)
        };
        assert!(same_rgb(style.visuals.panel_fill, opaque_panel));
        assert!(same_rgb(style.visuals.window_fill, opaque_window));
    }

    #[test]
    fn default_style_resets_to_fluent_metrics() {
        let ctx = egui::Context::default();
        let spacing = || {
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
        let fluent = (
            egui::vec2(10.0, 4.0),
            egui::vec2(8.0, 4.0),
            egui::Margin::same(6),
        );

        style::apply(&ctx, style::DEFAULT);
        assert_eq!(spacing(), fluent);

        style::apply(&ctx, style::COMPACT);
        assert_ne!(spacing(), fluent);
        style::apply(&ctx, style::DEFAULT);
        assert_eq!(spacing(), fluent);

        style::apply(&ctx, style::SPACIOUS);
        assert_ne!(spacing(), fluent);
        style::apply(&ctx, style::DEFAULT);
        assert_eq!(spacing(), fluent);
    }

    #[test]
    fn density_styles_keep_the_fluent_base_metrics() {
        let ctx = egui::Context::default();
        for density in style::all_names() {
            style::apply(&ctx, density);
            ctx.all_styles_mut(|s| {
                assert_eq!(
                    s.spacing.interact_size.y,
                    style::CONTROL_HEIGHT,
                    "{density}"
                );
                assert_eq!(s.spacing.slider_rail_height, 4.0, "{density}");
                assert_eq!(s.spacing.icon_width, style::ICON_WIDTH, "{density}");
                assert_eq!(
                    s.spacing.icon_width_inner,
                    style::ICON_WIDTH_INNER,
                    "{density}"
                );
                assert_eq!(s.spacing.icon_spacing, style::ICON_SPACING, "{density}");
                assert_eq!(
                    s.spacing.window_margin,
                    egui::Margin::same(style::WINDOW_MARGIN)
                );
                assert!(!s.spacing.scroll.floating || s.spacing.scroll.foreground_color);
            });
        }
    }
}
