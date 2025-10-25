// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::*;

mod eval;
use eval::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            inner_size: Some(Vec2::new(550., 445.)),
            min_inner_size: Some(Vec2::new(100., 100.)),
            icon: Some(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap()
                    .into(),
            ),
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
            .get_element_by_id("lapis_canvas")
            .expect("Failed to find lapis_canvas")
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
        let mut layouter = |ui: &Ui, buf: &dyn TextBuffer, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme_copy,
                buf.as_str(),
                "rs",
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts_mut(|f| f.layout_job(layout_job))
        };
        if self.keys_active {
            ctx.input(|i| {
                for event in &i.events {
                    if let Event::Key { key, modifiers, pressed, repeat, .. } = event {
                        if *repeat && !self.keys_repeat {
                            continue;
                        }
                        if let Some(code) = self.keys.get(&(*modifiers, *key, *pressed)) {
                            if self.quiet {
                                self.quiet_eval(&code.clone());
                            } else {
                                self.eval(&code.clone());
                            }
                        }
                    }
                }
            });
        }
        TopBottomPanel::bottom("input")
            .resizable(true)
            .show_separator_line(false)
            .min_height(90.)
            .show(ctx, |ui| {
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                            let execute = ui.button("e");
                            let input_focused = ui
                                .add(
                                    TextEdit::multiline(&mut self.input)
                                        .hint_text("type code then press ctrl+enter")
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
                                self.eval_input();
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
                ui.horizontal(|ui| {
                    ui.label("repo:");
                    ui.hyperlink_to(
                        "codeberg.org/tomara-x/lapis",
                        "https://codeberg.org/tomara-x/lapis/",
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("mirror:");
                    ui.hyperlink_to(
                        "github.com/tomara-x/lapis",
                        "https://github.com/tomara-x/lapis/",
                    );
                });
                ui.label(format!(
                    "version: {} ({})",
                    env!("CARGO_PKG_VERSION"),
                    env!("COMMIT_HASH")
                ));
                ui.horizontal(|ui| {
                    ui.label("FunDSP version:");
                    ui.hyperlink_to("tomara-x/fundsp", "https://codeberg.org/tomara-x/fundsp/");
                });
                ui.small("this machine kills fascists");
            });
            ui.horizontal(|ui| {
                if ui.button("settings").clicked() {
                    self.settings = !self.settings;
                }
                if ui.button("sliders").clicked() {
                    self.sliders_window = !self.sliders_window;
                }
                if ui.button("about").clicked() {
                    self.about = !self.about;
                }
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    ui.toggle_value(&mut self.keys_repeat, "keys repeat?")
                        .on_hover_text("enable key repeat events");
                    ui.toggle_value(&mut self.keys_active, "keys?")
                        .on_hover_text("enable key bindings");
                    ui.toggle_value(&mut self.quiet, "quiet?")
                        .on_hover_text("don't log keybinding evaluation");
                });
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            Window::new("settings").open(&mut self.settings).pivot(center).show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        theme.ui(ui);
                        theme.store_in_memory(ui.ctx());
                    });
                    ui.horizontal(|ui| {
                        ui.label("zoom factor");
                        ui.add(DragValue::new(&mut self.zoom_factor).range(0.5..=4.).speed(0.1));
                        ctx.set_zoom_factor(self.zoom_factor);
                    });
                });
            });
            Window::new("sliders").open(&mut self.sliders_window).pivot(center).show(ctx, |ui| {
                if ui.button("+").clicked() {
                    self.sliders.push(SliderSettings {
                        min: -1.,
                        max: 1.,
                        speed: 0.1,
                        step_by: 0.,
                        var: "".to_string(),
                    });
                }
                ScrollArea::vertical().show(ui, |ui| {
                    for s in &mut self.sliders {
                        ui.horizontal(|ui| {
                            ui.add(
                                TextEdit::singleline(&mut s.var)
                                    .hint_text("variable")
                                    .desired_width(80.),
                            )
                            .on_hover_text("the variable linked to this slider (float or shared)");
                            let mut tmp = 0.;
                            if let Some(v) = self.fmap.get(&s.var) {
                                tmp = *v as f32;
                            } else if let Some(v) = self.smap.get(&s.var) {
                                tmp = v.value();
                            }
                            ui.add(
                                Slider::new(&mut tmp, s.min..=s.max)
                                    .step_by(s.step_by)
                                    .drag_value_speed(s.speed),
                            );
                            ui.add(DragValue::new(&mut s.min).range(-1e6..=1e6))
                                .on_hover_text("min");
                            ui.add(DragValue::new(&mut s.max).range(-1e6..=1e6))
                                .on_hover_text("max");
                            ui.add(DragValue::new(&mut s.speed).range(0.0000001..=1.))
                                .on_hover_text("speed when dragging the number");
                            ui.add(DragValue::new(&mut s.step_by).range(0. ..=1.))
                                .on_hover_text("step size when dragging the slider (0 to disable)");
                            if let Some(v) = self.fmap.get_mut(&s.var) {
                                *v = tmp as f64;
                            } else if let Some(v) = self.smap.get(&s.var) {
                                v.set(tmp);
                            }
                        });
                    }
                });
            });
            ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(&mut self.buffer)
                        .font(TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(1)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
    }
}
