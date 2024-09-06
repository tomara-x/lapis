use eframe::egui;

mod audio;
mod components;
mod eval;
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
        let center = egui::Align2::CENTER_CENTER;
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
        egui::TopBottomPanel::top("top_panel").show_separator_line(false).show(ctx, |ui| {
            egui::Window::new("about").open(&mut self.about).pivot(center).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("lapis is a");
                    ui.hyperlink_to("fundsp",
                        "https://github.com/SamiPerttu/fundsp/");
                    ui.label("interpreter");
                });
                ui.label("an amy universe piece");
                ui.label("courtesy of the alphabet mafia");
                ui.horizontal(|ui| {
                    ui.label("repo:");
                    ui.hyperlink_to("github.com/tomara-x/lapis",
                        "https://github.com/tomara-x/lapis/");
                });
                ui.horizontal(|ui| {
                    ui.label("mirror:");
                    ui.hyperlink_to("codeberg.org/tomara-x/lapis",
                        "https://codeberg.org/tomara-x/lapis/");
                });
                let version = format!("{} {}", env!("CARGO_PKG_VERSION"), env!("COMMIT_HASH"));
                ui.label(format!("version: {}", version));
            });

            egui::Window::new("maps").open(&mut self.maps).pivot(center).show(ctx, |ui| {
                ui.group(|ui| {
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
            });
            ui.horizontal(|ui| {
                if ui.button("settings").clicked() {
                    self.settings = !self.settings;
                }
                if ui.button("maps").clicked() {
                    self.maps = !self.maps;
                }
                if ui.button("about").clicked() {
                    self.about = !self.about;
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Window::new("settings").open(&mut self.settings).pivot(center).show(ctx, |ui| {
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
