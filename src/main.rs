use eframe::egui;
use syn::Expr;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native("awawawa", options, Box::new(|_| Ok(Box::<Lapis>::default())))
}

struct Lapis {
    buffer: String,
    input: String,
    settings: bool,
}

impl Default for Lapis {
    fn default() -> Self {
        Self { buffer: "".into(), input: "".into(), settings: false }
    }
}

impl eframe::App for Lapis {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("input").show(ctx, |ui| {
            let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());

            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job =
                    egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "rs");
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };
            let input_focused = ui
                .add(
                    egui::TextEdit::multiline(&mut self.input)
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
                if let Ok(expr) = syn::parse_str::<Expr>(&self.input) {
                    self.buffer.push('\n');
                    self.buffer.push_str(&self.input);
                    println!("{:#?}", expr);
                }
                self.input.clear();
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
            if ui.button("settings").clicked() {
                self.settings = !self.settings;
            }
            egui::Window::new("settings").open(&mut self.settings).show(ctx, |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job =
                    egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "rs");
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

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
