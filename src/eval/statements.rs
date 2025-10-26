use crate::eval::*;
use cpal::traits::{DeviceTrait, HostTrait};
use crossbeam_channel::bounded;
use std::{thread, time::Duration};

pub fn eval_stmt(s: Stmt, lapis: &mut Lapis) -> String {
    let mut buffer = String::new();
    match s {
        Stmt::Local(expr) => {
            eval_local(&expr, lapis);
        }
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
            "\n// Sequencer(outs: {}, ins: {}, has_backend: {}, replay: {}, loop: ({}, {}))",
            seq.outputs(),
            seq.inputs(),
            seq.has_backend(),
            seq.replay_events(),
            seq.loop_start(),
            seq.loop_end(),
        );
        buffer.push_str(&info);
    } else if let Some(source) = eval_source(&expr, lapis) {
        buffer.push_str(&format!("\n// {:?}", source));
    } else if let Some(event) = eval_eventid(&expr, lapis) {
        buffer.push_str(&format!("\n// {:?}", event));
    } else if let Some(string) = eval_string(&expr, lapis) {
        buffer.push_str(&format!("\n// \"{}\"", string));
    } else if let Expr::Call(expr) = expr {
        function_calls(expr, lapis, buffer);
    } else if let Expr::Binary(expr) = expr {
        float_bin_assign(&expr, lapis);
    } else if let Expr::Break(_) = expr {
        buffer.push_str("#B");
    } else if let Expr::Continue(_) = expr {
        buffer.push_str("#C");
    } else if let Expr::MethodCall(expr) = expr {
        match expr.method.to_string().as_str() {
            "play" => {
                if let Some(mut g) = eval_net(&expr.receiver, lapis) {
                    let slot_outputs = lapis.slot.outputs();
                    if g.inputs() == 0
                        && g.outputs() == slot_outputs
                        && let Some((config, _)) = &lapis.out_stream
                    {
                        g.allocate();
                        g.set_sample_rate(config.sample_rate.0 as f64);
                        lapis.slot.set(Fade::Smooth, 0.01, Box::new(g));
                    }
                }
            }
            "drop" => {
                if let Some(k) = nth_path_ident(&expr.receiver, 0) {
                    lapis.drop(&k);
                }
            }
            "error" => {
                if let Some(k) = nth_path_ident(&expr.receiver, 0)
                    && let Some(g) = lapis.gmap.get_mut(&k)
                {
                    buffer.push_str(&format!("\n// {:?}", g.error()));
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

fn eval_local(expr: &Local, lapis: &mut Lapis) -> Option<()> {
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
            } else if let Some(table) = eval_atomic_table(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.atomic_table_map.insert(k, Arc::new(table));
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
            } else if let Some(string) = eval_string(&expr.expr, lapis) {
                lapis.drop(&k);
                lapis.string_map.insert(k, string);
            }
        }
    } else if let Pat::Tuple(pat) = &expr.pat
        && let Some(init) = &expr.init
        && let Expr::Call(call) = &*init.expr
    {
        let f = nth_path_ident(&call.func, 0)?;
        if f == "bounded" {
            let p0 = pat_ident(pat.elems.first()?)?;
            let p1 = pat_ident(pat.elems.get(1)?)?;
            let cap = eval_usize(call.args.first()?, lapis)?;
            let (s, r) = bounded(cap.clamp(0, 1000000));
            let s = Net::wrap(Box::new(An(BuffIn::new(s))));
            let r = Net::wrap(Box::new(An(BuffOut::new(r))));
            lapis.drop(&p0);
            lapis.gmap.insert(p0, s);
            lapis.drop(&p1);
            lapis.gmap.insert(p1, r);
        } else if f == "buffer" {
            let p0 = pat_ident(pat.elems.first()?)?;
            let p1 = pat_ident(pat.elems.get(1)?)?;
            let cap = eval_usize(call.args.first()?, lapis)?;
            // unlike bounded, you never need more than 64 here. like ever.. right?
            let (s, r) = fundsp::misc_nodes::buffer(cap.clamp(0, 1000000));
            let s = Net::wrap(Box::new(s));
            let r = Net::wrap(Box::new(r));
            lapis.drop(&p0);
            lapis.gmap.insert(p0, s);
            lapis.drop(&p1);
            lapis.gmap.insert(p1, r);
        } else if f == "Net" {
            let f = nth_path_ident(&call.func, 1)?;
            if f == "wrap_id" {
                let p0 = pat_ident(pat.elems.first()?)?;
                let p1 = pat_ident(pat.elems.get(1)?)?;
                let initial = eval_net(call.args.first()?, lapis)?;
                let (net, id) = Net::wrap_id(Box::new(initial));
                lapis.drop(&p0);
                lapis.gmap.insert(p0, net);
                lapis.drop(&p1);
                lapis.idmap.insert(p1, id);
            }
        }
    }
    None
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
            } else if let Some(event) = eval_eventid(&expr.right, lapis)
                && let Some(var) = lapis.eventmap.get_mut(&ident)
            {
                *var = event;
            } else if let Some(string) = eval_string(&expr.right, lapis)
                && let Some(var) = lapis.string_map.get_mut(&ident)
            {
                *var = string;
            }
        }
        Expr::Index(left) => {
            if let Some(k) = nth_path_ident(&left.expr, 0)
                && let Some(index) = eval_usize(&left.index, lapis)
                && let Some(right) = eval_float_f32(&expr.right, lapis)
                && let Some(vec) = lapis.vmap.get_mut(&k)
                && let Some(v) = vec.get_mut(index)
            {
                *v = right;
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
                } else if let Expr::Lit(right) = &*expr.right
                    && let Some(shortcut) = parse_shortcut(left.value())
                {
                    lapis.keys.remove(&shortcut);
                    if let Lit::Str(right) = &right.lit {
                        let key = shortcut.1.name();
                        let code = right.value().replace("$key", key);
                        if !code.is_empty() {
                            lapis.keys.insert(shortcut, code);
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
            lapis.fmap.insert(ident.clone(), i as f64);
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
            lapis.fmap.insert(ident.clone(), i as f64);
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

fn function_calls(expr: ExprCall, lapis: &mut Lapis, buffer: &mut String) -> Option<()> {
    let func = nth_path_ident(&expr.func, 0)?;
    match func.as_str() {
        "list_in_devices" => {
            let hosts = cpal::platform::ALL_HOSTS;
            buffer.push_str("\n// input devices:\n");
            for (i, host) in hosts.iter().enumerate() {
                buffer.push_str(&format!("// {}: {:?}:\n", i, host));
                if let Ok(devices) = cpal::platform::host_from_id(*host).unwrap().input_devices() {
                    for (j, device) in devices.enumerate() {
                        buffer.push_str(&format!("//     {}: {:?}\n", j, device.name()));
                    }
                }
            }
        }
        "list_out_devices" => {
            let hosts = cpal::platform::ALL_HOSTS;
            buffer.push_str("\n// output devices:\n");
            for (i, host) in hosts.iter().enumerate() {
                buffer.push_str(&format!("// {}: {:?}:\n", i, host));
                if let Ok(devices) = cpal::platform::host_from_id(*host).unwrap().output_devices() {
                    for (j, device) in devices.enumerate() {
                        buffer.push_str(&format!("//     {}: {:?}\n", j, device.name()));
                    }
                }
            }
        }
        "set_in_device" => {
            let h = eval_usize(expr.args.first()?, lapis);
            let d = eval_usize(expr.args.get(1)?, lapis);
            let channels = eval_usize(expr.args.get(2)?, lapis).map(|x| x as u16);
            let sr = eval_usize(expr.args.get(3)?, lapis).map(|x| x as u32);
            let buffer = eval_usize(expr.args.get(4)?, lapis).map(|x| x as u32);
            lapis.set_in_device(h, d, channels, sr, buffer);
        }
        "set_out_device" => {
            let h = eval_usize(expr.args.first()?, lapis);
            let d = eval_usize(expr.args.get(1)?, lapis);
            let channels = eval_usize(expr.args.get(2)?, lapis).map(|x| x as u16);
            let sr = eval_usize(expr.args.get(3)?, lapis).map(|x| x as u32);
            let buffer = eval_usize(expr.args.get(4)?, lapis).map(|x| x as u32);
            lapis.set_out_device(h, d, channels, sr, buffer);
        }
        "add_slider" => {
            let var = eval_str_lit(expr.args.first()?)?;
            let min = eval_float_f32(expr.args.get(1)?, lapis)?;
            let max = eval_float_f32(expr.args.get(2)?, lapis)?;
            let speed = eval_float(expr.args.get(3)?, lapis)?;
            let step_by = eval_float(expr.args.get(4)?, lapis)?;
            lapis.sliders.push(SliderSettings { min, max, speed, step_by, var });
        }
        "drop_in_stream" => lapis.in_stream = None,
        "drop_out_stream" => lapis.out_stream = None,
        "sleep" => {
            let d = eval_float(expr.args.first()?, lapis)?;
            let d = Duration::try_from_secs_f64(d).ok()?;
            thread::sleep(d);
        }
        "panic" => panic!(),
        "eval" => {
            let code = eval_string(expr.args.first()?, lapis)?;
            lapis.eval(&code);
        }
        "quiet_eval" => {
            let code = eval_string(expr.args.first()?, lapis)?;
            lapis.quiet_eval(&code);
        }
        _ => {}
    }
    None
}
