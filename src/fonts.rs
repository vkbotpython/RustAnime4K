//! UI font fallbacks.
//!
//! egui's bundled proportional font (Ubuntu-Light) covers Latin/Cyrillic
//! but lacks arrows and geometric shapes (`→ ▶ ↑ ↓`), while the media and
//! menu icons (`⏮ ⏭ ⏸ ⛶ 🎵 💬 ☰`) live in the monochrome emoji fonts
//! (enabled via the `monochrome_emoji_fonts` feature on `egui`).
//! Without both, labels and transport buttons render as tofu boxes.
//!
//! The bundled monospace font (Hack) does contain the arrows/shapes, and it
//! is already embedded by `default_fonts`, so it is added as a proportional
//! fallback: pure fallback, no extra binary size, normal text still renders
//! in Ubuntu-Light.

/// Patched font definitions: defaults plus Hack as a proportional fallback
/// (right after the primary font, before the emoji fallbacks).
pub(crate) fn ui_font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    if !fonts.font_data.contains_key("Hack") {
        return fonts;
    }
    if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        let hack = "Hack".to_owned();
        if !proportional.contains(&hack) {
            let pos = 1.min(proportional.len());
            proportional.insert(pos, hack);
        }
    }
    fonts
}

/// Installs [`ui_font_definitions`] on the context. Call once at startup.
pub(crate) fn install_ui_fonts(ctx: &egui::Context) {
    ctx.set_fonts(ui_font_definitions());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hack_is_proportional_fallback() {
        let fonts = ui_font_definitions();
        let proportional = &fonts.families[&egui::FontFamily::Proportional];
        assert_eq!(
            proportional.first().map(String::as_str),
            Some("Ubuntu-Light")
        );
        assert!(
            proportional.contains(&"Hack".to_owned()),
            "Hack must back proportional text (arrows/shapes)"
        );
    }

    #[test]
    fn emoji_fonts_are_installed() {
        // Transport/menu icons (media keys, notes, bubbles) need these.
        let fonts = ui_font_definitions();
        assert!(
            fonts.font_data.contains_key("NotoEmoji-Regular"),
            "monochrome_emoji_fonts feature must be enabled on egui"
        );
        assert!(fonts.font_data.contains_key("emoji-icon-font"));
    }
}
