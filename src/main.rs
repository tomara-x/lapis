use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui::*;

mod audio;
mod components;
mod eval;
use {audio::*, components::*, eval::*};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder {
            inner_size: Some(Vec2::new(550., 440.)),
            min_inner_size: Some(Vec2::new(100., 100.)),
            ..Default::default()
        },
        centered: true,
        ..Default::default()
    };
    eframe::run_native("awawawa", options, Box::new(|_| Ok(Box::new(Lapis::new()))))
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
        let mut in_device_change = false;
        let mut out_device_change = false;
        TopBottomPanel::bottom("input").resizable(true).show_separator_line(false).show(
            ctx,
            |ui| {
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    let input_focused = ui
                        .add(
                            TextEdit::multiline(&mut self.input)
                                .hint_text("type code then press ctrl+enter")
                                .font(TextStyle::Monospace)
                                .code_editor()
                                .desired_rows(1)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter),
                        )
                        .has_focus();
                    let shortcut =
                        KeyboardShortcut { modifiers: Modifiers::COMMAND, logical_key: Key::Enter };
                    if input_focused && ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                        eval(self);
                    }
                });
            },
        );
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
                let version = format!("{} {}", env!("CARGO_PKG_VERSION"), env!("COMMIT_HASH"));
                ui.label(format!("version: {}", version));
            });
            ui.horizontal(|ui| {
                if ui.button("settings").clicked() {
                    self.settings = !self.settings;
                }
                if ui.button("about").clicked() {
                    self.about = !self.about;
                }
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            Window::new("settings").open(&mut self.settings).pivot(center).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            theme.ui(ui);
                            theme.clone().store_in_memory(ui.ctx());
                        });
                    });
                    // TODO(amy): cleanup!
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.group(|ui| {
                                ui.label("input");
                                let host_change = ComboBox::from_id_salt("in_host")
                                    .show_index(ui, &mut self.in_host, cpal::ALL_HOSTS.len(), |i| {
                                        if let Some(host) = cpal::ALL_HOSTS.get(i) {
                                            host.name()
                                        } else {
                                            cpal::default_host().id().name()
                                        }
                                    })
                                    .changed();
                                if host_change {
                                    let i = self.in_host;
                                    let host = if let Some(host) = cpal::ALL_HOSTS.get(i) {
                                        cpal::host_from_id(*host).unwrap_or(cpal::default_host())
                                    } else {
                                        cpal::default_host()
                                    };
                                    if let Ok(devices) = host.input_devices() {
                                        self.in_device_names.clear();
                                        for device in devices {
                                            self.in_device_names
                                                .push(device.name().unwrap_or(String::from("Err")));
                                        }
                                    }
                                }
                                in_device_change = ComboBox::from_id_salt("in_device")
                                    .show_index(
                                        ui,
                                        &mut self.in_device,
                                        self.in_device_names.len(),
                                        |i| {
                                            if let Some(device) = self.in_device_names.get(i) {
                                                device.clone()
                                            } else if let Some(device) =
                                                cpal::default_host().default_output_device()
                                            {
                                                device.name().unwrap_or(String::from("Err"))
                                            } else {
                                                String::from("Err")
                                            }
                                        },
                                    )
                                    .changed();
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.group(|ui| {
                                ui.label("output");
                                let host_change = ComboBox::from_id_salt("out_host")
                                    .show_index(
                                        ui,
                                        &mut self.out_host,
                                        cpal::ALL_HOSTS.len(),
                                        |i| {
                                            if let Some(host) = cpal::ALL_HOSTS.get(i) {
                                                host.name()
                                            } else {
                                                cpal::default_host().id().name()
                                            }
                                        },
                                    )
                                    .changed();
                                if host_change {
                                    let i = self.out_host;
                                    let host = if let Some(host) = cpal::ALL_HOSTS.get(i) {
                                        cpal::host_from_id(*host).unwrap_or(cpal::default_host())
                                    } else {
                                        cpal::default_host()
                                    };
                                    if let Ok(devices) = host.output_devices() {
                                        self.out_device_names.clear();
                                        for device in devices {
                                            self.out_device_names
                                                .push(device.name().unwrap_or(String::from("Err")));
                                        }
                                    }
                                }
                                out_device_change = ComboBox::from_id_salt("out_device")
                                    .show_index(
                                        ui,
                                        &mut self.out_device,
                                        self.out_device_names.len(),
                                        |i| {
                                            if let Some(device) = self.out_device_names.get(i) {
                                                device.clone()
                                            } else if let Some(device) =
                                                cpal::default_host().default_output_device()
                                            {
                                                device.name().unwrap_or(String::from("Err"))
                                            } else {
                                                String::from("Err")
                                            }
                                        },
                                    )
                                    .changed();
                            });
                        });
                    });
                });
            });
            ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(&mut self.buffer)
                        .font(TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(2)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
        if in_device_change {
            set_in_device(self);
        }
        if out_device_change {
            set_out_device(self);
        }
    }
}
