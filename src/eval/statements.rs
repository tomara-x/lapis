use crate::eval::*;

pub fn eval_stmt(s: Stmt, lapis: &mut Lapis) -> String {
    let mut buffer = String::new();
    match s {
        Stmt::Local(expr) => eval_local(&expr, lapis),
        Stmt::Expr(expr, _) => match expr {
            Expr::Assign(expr) => eval_assign(&expr, lapis),
            Expr::ForLoop(expr) => eval_for_loop(&expr, lapis, &mut buffer),
            Expr::Block(expr) => eval_block(expr, lapis, &mut buffer),
            Expr::If(expr) => eval_if(expr, lapis, &mut buffer),
            expr => eval_expr(expr, lapis, &mut buffer),
        },
        _ => {}
    }
    buffer
}

fn eval_expr(expr: Expr, lapis: &mut Lapis, buffer: &mut String) {
    if let Some(n) = eval_float(&expr, lapis) {
        buffer.push_str(&format!("\n// {:?}", n));
    } else if let Some(arr) = eval_vec(&expr, lapis) {
        buffer.push_str(&format!("\n// {:?}", arr));
    } else if let Some(mut g) = eval_net_cloned(&expr, lapis) {
        let info = g.display().replace('\n', "\n// ");
        buffer.push_str(&format!("\n// {}", info));
        buffer.push_str(&format!("Size           : {}", g.size()));
    } else if let Some(id) = eval_nodeid(&expr, lapis) {
        buffer.push_str(&format!("\n// {:?}", id));
    } else if let Some(b) = eval_bool(&expr, lapis) {
        buffer.push_str(&format!("\n// {:?}", b));
    } else if let Some(s) = eval_shared(&expr, lapis) {
        buffer.push_str(&format!("\n// Shared({})", s.value()));
    } else if let Some(w) = path_wave(&expr, lapis) {
        buffer.push_str(&format!(
            "\n// Wave(ch:{}, sr:{}, len:{}, dur:{})",
            w.channels(),
            w.sample_rate(),
            w.len(),
            w.duration()
        ));
    } else if let Some(w) = eval_wave(&expr, lapis) {
        buffer.push_str(&format!(
            "\n// Wave(ch:{}, sr:{}, len:{}, dur:{})",
            w.channels(),
            w.sample_rate(),
            w.len(),
            w.duration()
        ));
    } else if let Some(seq) = path_seq(&expr, lapis).or(call_seq(&expr, lapis).as_ref()) {
        let info = format!(
            "\n// Sequencer(outs: {}, has_backend: {}, replay: {})",
            seq.outputs(),
            seq.has_backend(),
            seq.replay_events()
        );
        buffer.push_str(&info);
    } else if let Some(source) = eval_source(&expr, lapis) {
        buffer.push_str(&format!("\n// {:?}", source));
    } else if let Some(event) = eval_eventid(&expr, lapis) {
        buffer.push_str(&format!("\n// {:?}", event));
    } else if let Expr::Call(expr) = expr {
        device_commands(expr, lapis, buffer);
    } else if let Expr::Binary(expr) = expr {
        float_bin_assign(&expr, lapis);
    } else if let Expr::Break(_) = expr {
        buffer.push_str("#B");
    } else if let Expr::Continue(_) = expr {
        buffer.push_str("#C");
    } else if let Expr::MethodCall(expr) = expr {
        match expr.method.to_string().as_str() {
            "play" => {
                if let Some(g) = eval_net(&expr.receiver, lapis) {
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
                let Some(in_arr) = eval_vec(input, lapis) else { return };
                let mut output = Vec::new();
                if let Some(k) = nth_path_ident(&expr.receiver, 0) {
                    if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                        if g.inputs() != in_arr.len() {
                            return;
                        }
                        output.resize(g.outputs(), 0.);
                        g.tick(&in_arr, &mut output);
                    }
                } else if let Some(mut g) = eval_net(&expr.receiver, lapis) {
                    if g.inputs() != in_arr.len() {
                        return;
                    }
                    output.resize(g.outputs(), 0.);
                    g.tick(&in_arr, &mut output);
                }
                if let Some(out) = expr.args.get(1) {
                    if let Some(k) = nth_path_ident(out, 0) {
                        if let Some(var) = lapis.vmap.get_mut(&k) {
                            *var = output;
                        }
                    }
                } else {
                    buffer.push_str(&format!("\n// {:?}", output));
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
            "drop" => {
                if let Some(k) = nth_path_ident(&expr.receiver, 0) {
                    lapis.drop(&k);
                }
            }
            "error" => {
                if let Some(k) = nth_path_ident(&expr.receiver, 0) {
                    if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                        buffer.push_str(&format!("\n// {:?}", g.error()));
                    }
                }
            }
            _ => {
                wave_methods(&expr, lapis);
                net_methods(&expr, lapis);
                vec_methods(&expr, lapis);
                shared_methods(&expr, lapis);
                seq_methods(&expr, lapis);
            }
        }
    }
}

fn eval_if(expr: ExprIf, lapis: &mut Lapis, buffer: &mut String) {
    if let Some(cond) = eval_bool(&expr.cond, lapis) {
        if cond {
            let expr =
                Expr::Block(ExprBlock { attrs: Vec::new(), label: None, block: expr.then_branch });
            let s = eval_stmt(Stmt::Expr(expr, None), lapis);
            buffer.push_str(&s);
        } else if let Some((_, else_branch)) = expr.else_branch {
            let s = eval_stmt(Stmt::Expr(*else_branch, None), lapis);
            buffer.push_str(&s);
        }
    }
}

fn eval_block(expr: ExprBlock, lapis: &mut Lapis, buffer: &mut String) {
    for stmt in expr.block.stmts {
        buffer.push_str(&eval_stmt(stmt, lapis));
    }
}

fn eval_local(expr: &Local, lapis: &mut Lapis) {
    if let Some(k) = pat_ident(&expr.pat) {
        if let Some(expr) = &expr.init {
            if let Some(v) = eval_float(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.fmap.insert(k, v);
            } else if let Some(v) = eval_net(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.gmap.insert(k, v);
            } else if let Some(arr) = eval_vec(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.vmap.insert(k, arr);
            } else if let Some(id) = eval_nodeid(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.idmap.insert(k, id);
            } else if let Some(b) = eval_bool(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.bmap.insert(k, b);
            } else if let Some(s) = eval_shared(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.smap.insert(k, s);
            } else if let Some(w) = eval_wave(&expr.expr, lapis) {
                lapis.drop(&k);
                let wave = Arc::new(w);
                lapis.wmap.insert(k, wave);
            } else if let Some(seq) = call_seq(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.seqmap.insert(k, seq);
            } else if let Some(source) = eval_source(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.srcmap.insert(k, source);
            } else if let Some(event) = eval_eventid(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.eventmap.insert(k, event);
            }
        }
    }
}

#[allow(clippy::map_entry)]
fn eval_assign(expr: &ExprAssign, lapis: &mut Lapis) {
    match &*expr.left {
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
            } else if let Some(id) = eval_nodeid(&expr.right, lapis) {
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
            } else if let Some(s) = eval_source(&expr.right, lapis) {
                if let Some(var) = lapis.srcmap.get_mut(&ident) {
                    *var = s;
                }
            } else if let Some(event) = eval_eventid(&expr.right, lapis) {
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
            if let Lit::Str(left) = &left.lit {
                if let Some(b) = eval_bool(&expr.right, lapis) {
                    match left.value().as_str() {
                        "keys" => lapis.keys_active = b,
                        "quiet" => lapis.quiet = b,
                        _ => {}
                    }
                } else if let Expr::Lit(right) = &*expr.right {
                    if let Some(shortcut) = parse_shortcut(left.value()) {
                        lapis.keys.retain(|x| x.0 != shortcut);
                        if let Lit::Str(right) = &right.lit {
                            let code = right.value();
                            if !code.is_empty() {
                                lapis.keys.push((shortcut, code));
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn eval_for_loop(expr: &ExprForLoop, lapis: &mut Lapis, buffer: &mut String) {
    let Some(ident) = pat_ident(&expr.pat) else { return };
    let bounds = range_bounds(&expr.expr, lapis);
    let arr = eval_vec(&expr.expr, lapis);
    let tmp = lapis.fmap.remove(&ident);
    if let Some((r0, r1)) = bounds {
        'main_loop: for i in r0..r1 {
            lapis.fmap.insert(ident.clone(), i as f32);
            for stmt in &expr.body.stmts {
                let s = eval_stmt(stmt.clone(), lapis);
                buffer.push_str(&s);
                // NOTE amy.. you've out lazied yourself (proud of you)
                if buffer.ends_with("#B") {
                    buffer.pop();
                    buffer.pop();
                    break 'main_loop;
                } else if buffer.ends_with("#C") {
                    buffer.pop();
                    buffer.pop();
                    continue 'main_loop;
                }
            }
        }
    } else if let Some(arr) = arr {
        'main_loop: for i in arr {
            lapis.fmap.insert(ident.clone(), i);
            for stmt in &expr.body.stmts {
                let s = eval_stmt(stmt.clone(), lapis);
                buffer.push_str(&s);
                if buffer.ends_with("#B") {
                    buffer.pop();
                    buffer.pop();
                    break 'main_loop;
                } else if buffer.ends_with("#C") {
                    buffer.pop();
                    buffer.pop();
                    continue 'main_loop;
                }
            }
        }
    }
    if let Some(old) = tmp {
        lapis.fmap.insert(ident, old);
    } else {
        lapis.fmap.remove(&ident);
    }
}
