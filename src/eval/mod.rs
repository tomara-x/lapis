use crate::components::*;
use fundsp::hacker32::*;
use std::sync::Arc;
use syn::*;

mod arrays;
mod atomics;
mod bools;
mod floats;
mod helpers;
mod ints;
mod nets;
mod sequencers;
mod units;
mod waves;
use {
    arrays::*, atomics::*, bools::*, floats::*, helpers::*, ints::*, nets::*, sequencers::*,
    waves::*,
};

pub fn eval(lapis: &mut Lapis) {
    lapis.buffer.push('\n');
    match parse_str::<Stmt>(&lapis.input) {
        Ok(stmt) => {
            lapis.buffer.push_str(&lapis.input);
            lapis.input.clear();
            //println!("{:#?}", stmt);
            eval_stmt(stmt, lapis);
        }
        Err(err) => {
            lapis.buffer.push_str(&format!("// error: {}", err));
        }
    }
}

pub fn eval_stmt(s: Stmt, lapis: &mut Lapis) {
    match s {
        Stmt::Local(expr) => {
            if let Some(k) = pat_ident(&expr.pat) {
                if let Some(expr) = expr.init {
                    if let Some(v) = eval_float(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.fmap.insert(k, v);
                    } else if let Some(v) = eval_net(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.gmap.insert(k, v);
                    } else if let Some(arr) = eval_vec(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.vmap.insert(k, arr);
                    } else if let Some(id) =
                        method_nodeid(&expr.expr, lapis).or(path_nodeid(&expr.expr, lapis))
                    {
                        remove_from_all_maps(&k, lapis);
                        lapis.idmap.insert(k, id);
                    } else if let Some(b) = eval_bool(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.bmap.insert(k, b);
                    } else if let Some(s) = eval_shared(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.smap.insert(k, s);
                    } else if let Some(w) = eval_wave(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        let wave = Arc::new(w);
                        lapis.wmap.insert(k, wave);
                    } else if let Some(seq) = call_seq(&expr.expr, lapis) {
                        remove_from_all_maps(&k, lapis);
                        lapis.seqmap.insert(k, seq);
                    } else if let Some(event) =
                        method_eventid(&expr.expr, lapis).or(path_eventid(&expr.expr, lapis))
                    {
                        remove_from_all_maps(&k, lapis);
                        lapis.eventmap.insert(k, event);
                    }
                }
            }
        }
        Stmt::Expr(expr, _) => match expr {
            Expr::MethodCall(ref method) => match method.method.to_string().as_str() {
                "play" => {
                    if let Some(g) = eval_net(&method.receiver, lapis) {
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
                    let Some(input) = method.args.first() else { return };
                    let Some(in_arr) = eval_vec_cloned(input, lapis) else { return };
                    let mut output = Vec::new();
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                            if g.inputs() != in_arr.len() {
                                return;
                            }
                            output.resize(g.outputs(), 0.);
                            g.tick(&in_arr, &mut output);
                        }
                    } else if let Some(mut g) = eval_net(&method.receiver, lapis) {
                        if g.inputs() != in_arr.len() {
                            return;
                        }
                        output.resize(g.outputs(), 0.);
                        g.tick(&in_arr, &mut output);
                    }
                    if let Some(out) = method.args.get(1) {
                        if let Some(k) = nth_path_ident(out, 0) {
                            if let Some(var) = lapis.vmap.get_mut(&k) {
                                *var = output;
                            }
                        }
                    } else {
                        lapis.buffer.push_str(&format!("\n// {:?}", output));
                    }
                }
                "play_backend" => {
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                            if !g.has_backend() {
                                let g = g.backend();
                                if g.inputs() == 0 && g.outputs() == 2 {
                                    lapis.slot.set(Fade::Smooth, 0.01, Box::new(g));
                                }
                            }
                        } else if let Some(seq) = &mut lapis.seqmap.get_mut(&k) {
                            if !seq.has_backend() {
                                let backend = seq.backend();
                                if backend.outputs() == 2 {
                                    lapis.slot.set(Fade::Smooth, 0.01, Box::new(backend));
                                }
                            }
                        }
                    }
                }
                "drop" => {
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        remove_from_all_maps(&k, lapis);
                    }
                }
                "error" => {
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                            lapis.buffer.push_str(&format!("\n// {:?}", g.error()));
                        }
                    }
                }
                "source" => {
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get(&k) {
                            let arg0 = method.args.first();
                            let arg1 = method.args.get(1);
                            if let (Some(arg0), Some(arg1)) = (arg0, arg1) {
                                let id = path_nodeid(arg0, lapis);
                                let chan = eval_usize(arg1, lapis);
                                if let (Some(id), Some(chan)) = (id, chan) {
                                    if g.contains(id) && chan < g.inputs_in(id) {
                                        lapis
                                            .buffer
                                            .push_str(&format!("\n// {:?}", g.source(id, chan)));
                                    }
                                }
                            }
                        }
                    }
                }
                "output_source" => {
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get(&k) {
                            let arg0 = method.args.first();
                            if let Some(arg0) = arg0 {
                                let chan = eval_usize(arg0, lapis);
                                if let Some(chan) = chan {
                                    lapis
                                        .buffer
                                        .push_str(&format!("\n// {:?}", g.output_source(chan)));
                                }
                            }
                        }
                    }
                }
                _ => {
                    if let Some(n) = method_call_float(method, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", n));
                        return;
                    } else if let Some(arr) = method_call_vec_ref(method, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", arr));
                        return;
                    } else if let Some(nodeid) = method_nodeid(&expr, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", nodeid));
                        return;
                    } else if let Some(event) = method_eventid(&expr, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", event));
                        return;
                    } else if let Some(mut g) = method_net(method, lapis) {
                        let info = g.display().replace('\n', "\n// ");
                        lapis.buffer.push_str(&format!("\n// {}", info));
                        lapis.buffer.push_str(&format!("Size           : {}", g.size()));
                        return;
                    }
                    wave_methods(method, lapis);
                    net_methods(method, lapis);
                    vec_methods(method, lapis);
                    shared_methods(method, lapis);
                    seq_methods(method, lapis);
                }
            },
            Expr::Assign(expr) => match *expr.left {
                Expr::Path(_) => {
                    let Some(ident) = nth_path_ident(&expr.left, 0) else { return };
                    if let Some(f) = eval_float(&expr.right, lapis) {
                        if let Some(var) = lapis.fmap.get_mut(&ident) {
                            *var = f;
                        }
                    } else if lapis.gmap.contains_key(&ident) {
                        if let Some(g) = eval_net(&expr.right, lapis) {
                            lapis.gmap.insert(ident, g);
                        }
                    } else if lapis.vmap.contains_key(&ident) {
                        if let Some(a) = eval_vec(&expr.right, lapis) {
                            lapis.vmap.insert(ident, a);
                        }
                    } else if let Some(id) =
                        method_nodeid(&expr.right, lapis).or(path_nodeid(&expr.right, lapis))
                    {
                        if let Some(var) = lapis.idmap.get_mut(&ident) {
                            *var = id;
                        }
                    } else if let Some(b) = eval_bool(&expr.right, lapis) {
                        if let Some(var) = lapis.bmap.get_mut(&ident) {
                            *var = b;
                        }
                    } else if let Some(s) = eval_shared(&expr.right, lapis) {
                        if let Some(var) = lapis.smap.get_mut(&ident) {
                            *var = s;
                        }
                    } else if let Some(event) =
                        method_eventid(&expr.right, lapis).or(path_eventid(&expr.right, lapis))
                    {
                        if let Some(var) = lapis.eventmap.get_mut(&ident) {
                            *var = event;
                        }
                    }
                }
                Expr::Index(left) => {
                    if let Some(k) = nth_path_ident(&left.expr, 0) {
                        if let Some(index) = eval_usize(&left.index, lapis) {
                            if let Some(right) = eval_float(&expr.right, lapis) {
                                if let Some(vec) = lapis.vmap.get_mut(&k) {
                                    if let Some(v) = vec.get_mut(index) {
                                        *v = right;
                                    }
                                }
                            }
                        }
                    }
                }
                Expr::Lit(left) => {
                    if let Lit::Str(left) = left.lit {
                        if let Expr::Block(ref block) = *expr.right {
                            if let Some(shortcut) = parse_shortcut(left.value()) {
                                lapis.keys.retain(|x| x.0 != shortcut);
                                if !block.block.stmts.is_empty() {
                                    let stmt = Stmt::Expr(*expr.right, None);
                                    lapis.keys.push((shortcut, stmt));
                                }
                            }
                        } else if let Some(b) = eval_bool(&expr.right, lapis) {
                            if left.value() == "keys" {
                                lapis.keys_active = b;
                            }
                        }
                    }
                }
                _ => {}
            },
            Expr::ForLoop(expr) => {
                let Some(ident) = pat_ident(&expr.pat) else { return };
                let bounds = range_bounds(&expr.expr, lapis);
                let arr = eval_vec(&expr.expr, lapis);
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
                if let Some(cond) = eval_bool(&expr.cond, lapis) {
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
                if let Some(n) = eval_float(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", n));
                } else if let Some(arr) = eval_vec_ref(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", arr));
                } else if let Some(arr) = eval_vec_cloned(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", arr));
                } else if let Some(mut g) = eval_net_cloned(&expr, lapis) {
                    let info = g.display().replace('\n', "\n// ");
                    lapis.buffer.push_str(&format!("\n// {}", info));
                    lapis.buffer.push_str(&format!("Size           : {}", g.size()));
                } else if let Some(id) = path_nodeid(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", id));
                } else if let Some(b) = eval_bool(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", b));
                } else if let Some(s) = eval_shared(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// Shared({})", s.value()));
                } else if let Some(w) = path_wave(&expr, lapis) {
                    lapis.buffer.push_str(&format!(
                        "\n// Wave(ch:{}, sr:{}, len:{}, dur:{})",
                        w.channels(),
                        w.sample_rate(),
                        w.len(),
                        w.duration()
                    ));
                } else if let Some(w) = eval_wave(&expr, lapis) {
                    lapis.buffer.push_str(&format!(
                        "\n// Wave(ch:{}, sr:{}, len:{}, dur:{})",
                        w.channels(),
                        w.sample_rate(),
                        w.len(),
                        w.duration()
                    ));
                } else if let Some(seq) = path_seq(&expr, lapis).or(call_seq(&expr, lapis).as_ref())
                {
                    let info = format!(
                        "\n// Sequencer(outs: {}, has_backend: {}, replay: {})",
                        seq.outputs(),
                        seq.has_backend(),
                        seq.replay_events()
                    );
                    lapis.buffer.push_str(&info);
                } else if let Some(event) = path_eventid(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", event));
                } else if let Expr::Call(expr) = expr {
                    device_commands(expr, lapis);
                }
            }
        },
        _ => {}
    }
}
