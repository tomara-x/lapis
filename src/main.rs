use eframe::egui;

mod audio;
mod components;
mod eval;
mod units;
use {components::*, eval::*};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 440.0]),
        ..Default::default()
    };
    eframe::run_native("awawawa", options, Box::new(|_| Ok(Box::new(Lapis::new()))))
}

impl eframe::App for Lapis {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ctx);
        let theme_copy = theme.clone();
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme_copy, string, "rs");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        egui::TopBottomPanel::bottom("input").resizable(true).show_separator_line(false).show(
            ctx,
            |ui| {
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    let input_focused = ui
                        .add(
                            egui::TextEdit::multiline(&mut self.input)
                                .hint_text("type code then press ctrl+enter")
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_rows(1)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter),
                        )
                        .has_focus();
                    let shortcut = egui::KeyboardShortcut {
                        modifiers: egui::Modifiers::COMMAND,
                        logical_key: egui::Key::Enter,
                    };
                    if input_focused && ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                        eval(self);
                    }
                });
            },
        );
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("settings").clicked() {
                    self.settings = !self.settings;
                }
                if ui.button("clear fmap").clicked() {
                    self.fmap.clear();
                    self.fmap.shrink_to_fit();
                }
                if ui.button("clear vmap").clicked() {
                    self.vmap.clear();
                    self.vmap.shrink_to_fit();
                }
                if ui.button("clear gmap").clicked() {
                    self.gmap.clear();
                    self.gmap.shrink_to_fit();
                }
            });
            egui::Window::new("settings").open(&mut self.settings).show(ctx, |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.buffer)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(2)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
    }
}
