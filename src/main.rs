// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::*;

mod audio;
mod components;
mod eval;
use {components::*, eval::*};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            inner_size: Some(Vec2::new(550., 445.)),
            min_inner_size: Some(Vec2::new(100., 100.)),
            ..Default::default()
        },
        centered: true,
        ..Default::default()
    };
    eframe::run_native("awawawa", options, Box::new(|_| Ok(Box::new(Lapis::new()))))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().expect("No window").document().expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(|_| Ok(Box::new(Lapis::new()))))
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

impl eframe::App for Lapis {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let center = Align2::CENTER_CENTER;
        let mut theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ctx, &ctx.style());
        let theme_copy = theme.clone();
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme_copy,
                string,
                "rs",
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        if self.keys_active {
            for (shortcut, stmt) in self.keys.clone() {
                if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                    eval_stmt(stmt, self);
                }
            }
        }
        TopBottomPanel::bottom("input")
            .resizable(true)
            .show_separator_line(false)
            .min_height(80.)
            .show(ctx, |ui| {
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                            let execute = ui.button("e");
                            let input_focused = ui
                                .add(
                                    TextEdit::multiline(&mut self.input)
                                        .hint_text("type a statement then press ctrl+enter")
                                        .font(TextStyle::Monospace)
                                        .code_editor()
                                        .desired_rows(5)
                                        .lock_focus(true)
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut layouter),
                                )
                                .has_focus();
                            let shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Enter);
                            if input_focused && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
                                || execute.clicked()
                            {
                                eval(self);
                            }
                        });
                    });
                });
            });
        TopBottomPanel::top("top_panel").show_separator_line(false).show(ctx, |ui| {
            Window::new("about").open(&mut self.about).pivot(center).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("lapis is a");
                    ui.hyperlink_to("FunDSP", "https://github.com/SamiPerttu/fundsp/");
                    ui.label("interpreter");
                });
                ui.label("an amy universe piece");
                ui.label("courtesy of the alphabet mafia");
                ui.horizontal(|ui| {
                    ui.label("repo:");
                    ui.hyperlink_to(
                        "github.com/tomara-x/lapis",
                        "https://github.com/tomara-x/lapis/",
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("mirror:");
                    ui.hyperlink_to(
                        "codeberg.org/tomara-x/lapis",
                        "https://codeberg.org/tomara-x/lapis/",
                    );
                });
                let version = format!("{} ({})", env!("CARGO_PKG_VERSION"), env!("COMMIT_HASH"));
                ui.label(format!("version: {}", version));
                ui.label("FunDSP version: master")
            });
            ui.horizontal(|ui| {
                if ui.button("settings").clicked() {
                    self.settings = !self.settings;
                }
                if ui.button("about").clicked() {
                    self.about = !self.about;
                }
                ui.checkbox(&mut self.keys_active, "keys");
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            Window::new("settings").open(&mut self.settings).pivot(center).show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        theme.ui(ui);
                        theme.store_in_memory(ui.ctx());
                    });
                });
            });
            ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(&mut self.buffer)
                        .font(TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(23)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
    }
}
