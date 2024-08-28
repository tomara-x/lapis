use eframe::egui;
use std::collections::HashMap;
use syn::*;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native("awawawa", options, Box::new(|_| Ok(Box::<Lapis>::default())))
}

#[derive(Default)]
struct Lapis {
    buffer: String,
    input: String,
    settings: bool,
    fmap: HashMap<String, f32>,
    //vmap: HashMap<String, Vec<f32>>,
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
                eval(self);
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

fn eval(lapis: &mut Lapis) {
    if let Ok(stmt) = parse_str::<Stmt>(&lapis.input) {
        lapis.buffer.push('\n');
        lapis.buffer.push_str(&lapis.input);
        lapis.input.clear();
        println!("{:#?}", stmt);
        if let Stmt::Expr(Expr::Block(expr), _) = stmt {
            for stmt in expr.block.stmts {
                eval_stmt(stmt, lapis);
            }
        } else {
            eval_stmt(stmt, lapis);
        }
    }
}

fn eval_stmt(s: Stmt, lapis: &mut Lapis) {
    match s {
        Stmt::Local(expr) => {
            if let Pat::Ident(i) = expr.pat {
                let k = i.ident.to_string();
                if let Some(expr) = expr.init {
                    let expr = *expr.expr;
                    let v = match expr {
                        Expr::Lit(expr) => lit_float(&expr.lit),
                        Expr::Binary(expr) => bin_expr_float(&expr),
                        Expr::Paren(expr) => paren_expr_float(&expr.expr),
                        _ => None,
                    };
                    if let Some(v) = v {
                        lapis.fmap.insert(k, v);
                    }
                }
            }
        }
        Stmt::Expr(expr, _) => match expr {
            Expr::Path(expr) => {
                let segments = &expr.path.segments;
                if let Some(s) = segments.first() {
                    let k = s.ident.to_string();
                    lapis.buffer.push_str(&format!("\n>{:?}", lapis.fmap.get(&k)));
                }
            }
            Expr::Binary(expr) => {
                println!("{:?}", bin_expr_float(&expr));
            }
            _ => {}
        },
        _ => {}
    }
}

fn bin_expr_float(expr: &ExprBinary) -> Option<f32> {
    let left = match *expr.left.clone() {
        Expr::Lit(expr) => lit_float(&expr.lit)?,
        Expr::Binary(expr) => bin_expr_float(&expr)?,
        Expr::Paren(expr) => paren_expr_float(&expr.expr)?,
        _ => return None,
    };
    let right = match *expr.right.clone() {
        Expr::Lit(expr) => lit_float(&expr.lit)?,
        Expr::Binary(expr) => bin_expr_float(&expr)?,
        Expr::Paren(expr) => paren_expr_float(&expr.expr)?,
        _ => return None,
    };
    match expr.op {
        BinOp::Sub(_) => Some(left - right),
        BinOp::Div(_) => Some(left / right),
        BinOp::Mul(_) => Some(left * right),
        BinOp::Add(_) => Some(left + right),
        _ => None,
    }
}

fn lit_float(expr: &Lit) -> Option<f32> {
    match expr {
        Lit::Float(expr) => expr.base10_parse::<f32>().ok(),
        Lit::Int(expr) => expr.base10_parse::<f32>().ok(),
        _ => None,
    }
}

fn paren_expr_float(expr: &Expr) -> Option<f32> {
    match expr {
        Expr::Lit(expr) => lit_float(&expr.lit),
        Expr::Binary(expr) => bin_expr_float(expr),
        _ => None,
    }
}
