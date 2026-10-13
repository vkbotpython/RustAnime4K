//! Диспетчер диалогов.

use crate::app::state::Anime4KPlayerApp;

mod about;
mod android;
mod help;
mod launch;
mod open;
mod preset_offer;
mod screenshot;
mod settings;
mod ui_kit;

impl Anime4KPlayerApp {
    pub(crate) fn dialogs(&mut self, ctx: &egui::Context) {
        self.poll_screenshot();
        self.settings_dialog(ctx);
        self.about_dialog(ctx);
        self.preset_offer_dialog(ctx);
        self.open_url_dialog(ctx);
        self.open_folder_dialog(ctx);
        self.screenshot_dialog(ctx);
        self.launch_params_dialog(ctx);
        self.android_dialog(ctx);
        self.reference_dialog(ctx);
    }
}
