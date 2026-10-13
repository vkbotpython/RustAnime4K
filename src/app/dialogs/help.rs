//! Диалог справки по Anime4K.

use crate::app::state::Anime4KPlayerApp;
use crate::i18n::Lang;

impl Anime4KPlayerApp {
    pub(crate) fn reference_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_reference {
            return;
        }
        let title = self.t("Reference");
        let russian = self.lang == Lang::Russian;
        let mut open = self.show_reference;
        egui::Window::new(title)
            .open(&mut open)
            .default_size([680.0, 560.0])
            .min_width(540.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        crate::reference::show(ui, russian);
                    });
            });
        self.show_reference = open;
    }
}
