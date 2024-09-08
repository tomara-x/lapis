use crate::components::*;
use fundsp::hacker32::*;
use syn::*;

mod arrays;
mod bools;
mod floats;
mod functions;
mod ints;
mod net_methods;
mod nets;
mod node_ids;
mod shapes;
mod units;
use {arrays::*, bools::*, floats::*, functions::*, net_methods::*, nets::*, node_ids::*};

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
            if let Some(k) = pat_ident(&expr.pat) {
                if let Some(expr) = expr.init {
                    if let Some(v) = half_binary_float(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.fmap.insert(k, v);
                    } else if let Some(v) = half_binary_net(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.gmap.insert(k, v);
                    } else if let Some(arr) = array_cloned(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.vmap.insert(k, arr);
                    } else if let Some(id) =
                        method_nodeid(&expr.expr, lapis).or(path_nodeid(&expr.expr, lapis))
                    {
                        remove_from_all_maps(&k, lapis);
                        lapis.idmap.insert(k, id);
                    } else if let Some(b) = half_binary_bool(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.bmap.insert(k, b);
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
                    if let Some(k) = nth_path_ident(&expr.receiver, 0) {
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
                        if let Some(k) = nth_path_ident(out, 0) {
                            lapis.vmap.insert(k, output);
                        }
                    }
                }
                "play_backend" => {
                    if let Some(k) = nth_path_ident(&expr.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                            if !g.has_backend() {
                                let g = g.backend();
                                if g.inputs() == 0 && g.outputs() == 2 {
                                    lapis.slot.set(Fade::Smooth, 0.01, Box::new(g));
                                }
                            }
                        }
                    }
                }
                _ => {
                    let _ = net_methods(&expr, lapis);
                    let _ = method_nodeid(&Expr::MethodCall(expr), lapis);
                }
            },
            Expr::Assign(expr) => {
                let Some(ident) = nth_path_ident(&expr.left, 0) else { return };
                if let Some(f) = half_binary_float(&expr.right, lapis) {
                    if let Some(var) = lapis.fmap.get_mut(&ident) {
                        *var = f;
                    }
                } else if let Some(g) = half_binary_net(&expr.right, lapis) {
                    if let Some(var) = lapis.gmap.get_mut(&ident) {
                        *var = g;
                    }
                } else if let Some(a) = array_cloned(&expr.right, lapis) {
                    if let Some(var) = lapis.vmap.get_mut(&ident) {
                        *var = a;
                    }
                } else if let Some(id) =
                    method_nodeid(&expr.right, lapis).or(path_nodeid(&expr.right, lapis))
                {
                    if let Some(var) = lapis.idmap.get_mut(&ident) {
                        *var = id;
                    }
                } else if let Some(b) = half_binary_bool(&expr.right, lapis) {
                    if let Some(var) = lapis.bmap.get_mut(&ident) {
                        *var = b;
                    }
                }
            }
            Expr::ForLoop(expr) => {
                let Some(ident) = pat_ident(&expr.pat) else { return };
                let bounds = range_bounds(&expr.expr);
                let arr = array_cloned(&expr.expr, lapis);
                let tmp = lapis.fmap.remove(&ident);
                if let Some((r0, r1)) = bounds {
                    for i in r0..r1 {
                        lapis.fmap.insert(ident.clone(), i as f32);
                        for stmt in &expr.body.stmts {
                            eval_stmt(stmt.clone(), lapis);
                        }
                    }
                } else if let Some(arr) = arr {
                    for i in arr {
                        lapis.fmap.insert(ident.clone(), i);
                        for stmt in &expr.body.stmts {
                            eval_stmt(stmt.clone(), lapis);
                        }
                    }
                }
                if let Some(old) = tmp {
                    lapis.fmap.insert(ident, old);
                } else {
                    lapis.fmap.remove(&ident);
                }
            }
            Expr::If(expr) => {
                if let Some(cond) = half_binary_bool(&expr.cond, lapis) {
                    if cond {
                        let expr = Expr::Block(ExprBlock {
                            attrs: Vec::new(),
                            label: None,
                            block: expr.then_branch,
                        });
                        eval_stmt(Stmt::Expr(expr, None), lapis);
                    } else if let Some((_, else_branch)) = expr.else_branch {
                        eval_stmt(Stmt::Expr(*else_branch, None), lapis);
                    }
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
                } else if let Some(id) = path_nodeid(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n    {:?}", id));
                } else if let Some(b) = half_binary_bool(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n    {:?}", b));
                }
            }
        },
        _ => {}
    }
}
