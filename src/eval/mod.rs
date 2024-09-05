use crate::components::*;
use fundsp::hacker32::*;
use syn::*;

mod arrays;
mod floats;
mod functions;
mod ints;
mod nets;
mod units;
use {arrays::*, floats::*, functions::*, nets::*};

pub fn eval(lapis: &mut Lapis) {
    if let Ok(stmt) = parse_str::<Stmt>(&lapis.input) {
        lapis.buffer.push('\n');
        lapis.buffer.push_str(&lapis.input);
        lapis.input.clear();
        println!("{:#?}", stmt);
        eval_stmt(stmt, lapis);
    }
}

fn eval_stmt(s: Stmt, lapis: &mut Lapis) {
    match s {
        Stmt::Local(expr) => {
            if let Pat::Ident(i) = expr.pat {
                let k = i.ident.to_string();
                if let Some(expr) = expr.init {
                    if let Some(v) = half_binary_float(&expr.expr, lapis) {
                        lapis.vmap.remove(&k);
                        lapis.gmap.remove(&k);
                        lapis.fmap.insert(k, v);
                    } else if let Some(v) = half_binary_net(&expr.expr, lapis) {
                        lapis.vmap.remove(&k);
                        lapis.fmap.remove(&k);
                        lapis.gmap.insert(k, v);
                    } else if let Some(arr) = array_lit(&expr.expr, lapis) {
                        lapis.fmap.remove(&k);
                        lapis.gmap.remove(&k);
                        lapis.vmap.insert(k, arr);
                    }
                }
            }
        }
        Stmt::Expr(expr, _) => match expr {
            Expr::MethodCall(expr) => match expr.method.to_string().as_str() {
                "play" => {
                    if let Some(g) = half_binary_net(&expr.receiver, lapis) {
                        if g.inputs() == 0 && g.outputs() == 1 {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(g | dc(0.)));
                        } else if g.inputs() == 0 && g.outputs() == 2 {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(g));
                        } else {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(dc(0.) | dc(0.)));
                        }
                    }
                }
                "tick" => {
                    let Some(input) = expr.args.first() else { return };
                    let Some(in_arr) = array_cloned(input, lapis) else { return };
                    let mut output = Vec::new();
                    if let Some(k) = path_ident(&expr.receiver) {
                        if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                            if g.inputs() != in_arr.len() {
                                return;
                            }
                            output.resize(g.outputs(), 0.);
                            g.tick(&in_arr, &mut output);
                        }
                    } else if let Some(mut g) = half_binary_net(&expr.receiver, lapis) {
                        if g.inputs() != in_arr.len() {
                            return;
                        }
                        output.resize(g.outputs(), 0.);
                        g.tick(&in_arr, &mut output);
                    }
                    lapis.buffer.push_str(&format!("\n    {:?}", output));
                    if let Some(out) = expr.args.get(1) {
                        if let Some(k) = path_ident(out) {
                            lapis.vmap.insert(k, output);
                        }
                    }
                }
                _ => {}
            },
            Expr::Assign(expr) => {
                let Some(ident) = path_ident(&expr.left) else { return };
                if let Some(f) = half_binary_float(&expr.right, lapis) {
                    if let Some(var) = lapis.fmap.get_mut(&ident) {
                        *var = f;
                    }
                } else if let Some(g) = half_binary_net(&expr.right, lapis) {
                    if let Some(var) = lapis.gmap.get_mut(&ident) {
                        *var = g;
                    }
                } else if let Some(a) = array_lit(&expr.right, lapis) {
                    if let Some(var) = lapis.vmap.get_mut(&ident) {
                        *var = a;
                    }
                }
            }
            Expr::ForLoop(expr) => {
                let Some(ident) = pat_ident(&expr.pat) else { return };
                let Some((r0, r1)) = range_bounds(&expr.expr) else { return };
                let tmp = lapis.fmap.remove(&ident);
                for i in r0..r1 {
                    lapis.fmap.insert(ident.clone(), i as f32);
                    for stmt in &expr.body.stmts {
                        eval_stmt(stmt.clone(), lapis);
                    }
                }
                if let Some(old) = tmp {
                    lapis.fmap.insert(ident, old);
                } else {
                    lapis.fmap.remove(&ident);
                }
            }
            Expr::Block(expr) => {
                for stmt in expr.block.stmts {
                    eval_stmt(stmt, lapis);
                }
            }
            _ => {
                if let Some(n) = half_binary_float(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n    {:?}", n));
                } else if let Some(arr) = path_arr(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n    {:?}", arr));
                } else if let Some(mut g) = half_binary_net(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n{}", g.display()));
                    lapis.buffer.push_str(&format!("Size           : {}", g.size()));
                }
            }
        },
        _ => {}
    }
}
