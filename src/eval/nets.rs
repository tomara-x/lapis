use crate::eval::*;
use fundsp::sound::*;

pub fn eval_net(expr: &Expr, lapis: &mut Lapis) -> Option<Net> {
    match expr {
        Expr::Call(expr) => call_net(expr, lapis),
        Expr::Binary(expr) => bin_expr_net(expr, lapis),
        Expr::Paren(expr) => eval_net(&expr.expr, lapis),
        Expr::Path(expr) => path_net(&expr.path, lapis),
        Expr::Unary(expr) => unary_net(expr, lapis),
        Expr::MethodCall(expr) => method_net(expr, lapis),
        _ => None,
    }
}

pub fn eval_net_cloned(expr: &Expr, lapis: &mut Lapis) -> Option<Net> {
    match expr {
        Expr::Call(expr) => call_net(expr, lapis),
        Expr::Binary(expr) => bin_expr_net(expr, lapis),
        Expr::Paren(expr) => eval_net(&expr.expr, lapis),
        Expr::Path(expr) => path_net_cloned(&expr.path, lapis),
        Expr::Unary(expr) => unary_net(expr, lapis),
        Expr::MethodCall(expr) => method_net(expr, lapis),
        _ => None,
    }
}

fn method_net(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<Net> {
    match expr.method.to_string().as_str() {
        "backend" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            if let Some(seq) = lapis.seqmap.get_mut(&k) {
                if !seq.has_backend() {
                    return Some(Net::wrap(Box::new(seq.backend())));
                }
            } else if let Some(g) = lapis.gmap.get_mut(&k)
                && !g.has_backend()
            {
                return Some(Net::wrap(Box::new(g.backend())));
            }
            None
        }
        "clone" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            lapis.gmap.get(&k).cloned()
        }
        "remove" => {
            let arg = expr.args.first()?;
            let id = eval_nodeid(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id) { Some(Net::wrap(net.remove(id))) } else { None }
        }
        "remove_link" => {
            let arg = expr.args.first()?;
            let id = eval_nodeid(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id) && net.inputs_in(id) == net.outputs_in(id) {
                return Some(Net::wrap(net.remove_link(id)));
            }
            None
        }
        "replace" => {
            let arg0 = expr.args.first()?;
            let id = eval_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let unit = eval_net(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id)
                && unit.inputs() == net.inputs_in(id)
                && unit.outputs() == net.outputs_in(id)
            {
                return Some(Net::wrap(net.replace(id, Box::new(unit))));
            }
            None
        }
        "phase" => {
            let p = eval_float_f32(expr.args.first()?, lapis)?;
            let mut net = eval_net(&expr.receiver, lapis)?;
            // bad amy
            for i in 0..net.ids().len() {
                net.set(Setting::phase(p).node(*net.ids().nth(i)?).right());
            }
            net.reset();
            Some(net)
        }
        "seed" => {
            let s = eval_u64(expr.args.first()?, lapis)?;
            let mut net = eval_net(&expr.receiver, lapis)?;
            // really bad amy
            for i in 0..net.ids().len() {
                net.set(Setting::seed(s).node(*net.ids().nth(i)?).left());
            }
            net.reset();
            Some(net)
        }
        _ => None,
    }
}

fn bin_expr_net(expr: &ExprBinary, lapis: &mut Lapis) -> Option<Net> {
    let left_net = eval_net(&expr.left, lapis);
    let right_net = eval_net(&expr.right, lapis);
    let left_float = eval_float_f32(&expr.left, lapis);
    let right_float = eval_float_f32(&expr.right, lapis);
    match (left_net, right_net, left_float, right_float) {
        (Some(left), Some(right), _, _) => {
            let (li, lo) = (left.inputs(), left.outputs());
            let (ri, ro) = (right.inputs(), right.outputs());
            match expr.op {
                BinOp::BitAnd(_) if li == ri && lo == ro => Some(left & right),
                BinOp::BitOr(_) => Some(left | right),
                BinOp::BitXor(_) if li == ri => Some(left ^ right),
                BinOp::Shr(_) if lo == ri => Some(left >> right),
                BinOp::Sub(_) if lo == ro => Some(left - right),
                BinOp::Mul(_) if lo == ro => Some(left * right),
                BinOp::Add(_) if lo == ro => Some(left + right),
                _ => None,
            }
        }
        (Some(left), _, _, Some(right)) => match expr.op {
            BinOp::Sub(_) => Some(left - right),
            BinOp::Mul(_) => Some(left * right),
            BinOp::Add(_) => Some(left + right),
            _ => None,
        },
        (_, Some(right), Some(left), _) => match expr.op {
            BinOp::Sub(_) => Some(left - right),
            BinOp::Mul(_) => Some(left * right),
            BinOp::Add(_) => Some(left + right),
            _ => None,
        },
        _ => None,
    }
}

fn unary_net(expr: &ExprUnary, lapis: &mut Lapis) -> Option<Net> {
    match expr.op {
        UnOp::Neg(_) => Some(-eval_net(&expr.expr, lapis)?),
        UnOp::Not(_) => Some(!eval_net(&expr.expr, lapis)?),
        _ => None,
    }
}

fn path_net(expr: &Path, lapis: &mut Lapis) -> Option<Net> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.gmap.remove(&k)
}

fn path_net_cloned(expr: &Path, lapis: &Lapis) -> Option<Net> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.gmap.get(&k).cloned()
}

pub fn net_methods(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<()> {
    match expr.method.to_string().as_str() {
        "remove" => {
            let arg = expr.args.first()?;
            let id = eval_nodeid(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id) {
                net.remove(id);
            }
        }
        "remove_link" => {
            let arg = expr.args.first()?;
            let id = eval_nodeid(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id) && net.inputs_in(id) == net.outputs_in(id) {
                net.remove_link(id);
            }
        }
        "replace" => {
            let arg0 = expr.args.first()?;
            let id = eval_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let unit = eval_net(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id)
                && unit.inputs() == net.inputs_in(id)
                && unit.outputs() == net.outputs_in(id)
            {
                net.replace(id, Box::new(unit));
            }
        }
        "crossfade" => {
            let arg0 = expr.args.first()?;
            let id = eval_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let fade = path_fade(arg1)?;
            let arg2 = expr.args.get(2)?;
            let time = eval_float_f32(arg2, lapis)?;
            let arg3 = expr.args.get(3)?;
            let unit = eval_net(arg3, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id)
                && unit.inputs() == net.inputs_in(id)
                && unit.outputs() == net.outputs_in(id)
            {
                net.crossfade(id, fade, time, Box::new(unit));
            }
        }
        "connect" => {
            let arg0 = expr.args.first()?;
            let src = eval_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let src_port = eval_usize(arg1, lapis)?;
            let arg2 = expr.args.get(2)?;
            let snk = eval_nodeid(arg2, lapis)?;
            if src == snk {
                return None;
            }
            let arg3 = expr.args.get(3)?;
            let snk_port = eval_usize(arg3, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(src) && net.contains(snk) {
                let src_outs = net.outputs_in(src);
                let snk_ins = net.inputs_in(snk);
                if src_port < src_outs && snk_port < snk_ins {
                    net.connect(src, src_port, snk, snk_port);
                }
            }
        }
        "disconnect" => {
            let arg0 = expr.args.first()?;
            let id = eval_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let port = eval_usize(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id) && port < net.inputs_in(id) {
                net.disconnect(id, port);
            }
        }
        "connect_input" => {
            let arg0 = expr.args.first()?;
            let global_in = eval_usize(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let snk = eval_nodeid(arg1, lapis)?;
            let arg2 = expr.args.get(2)?;
            let snk_port = eval_usize(arg2, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if global_in < net.inputs() && net.contains(snk) && snk_port < net.inputs_in(snk) {
                net.connect_input(global_in, snk, snk_port);
            }
        }
        "pipe_input" => {
            let arg0 = expr.args.first()?;
            let snk = eval_nodeid(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(snk) {
                net.pipe_input(snk);
            }
        }
        "connect_output" => {
            let arg0 = expr.args.first()?;
            let src = eval_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let src_port = eval_usize(arg1, lapis)?;
            let arg2 = expr.args.get(2)?;
            let global_out = eval_usize(arg2, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if global_out < net.outputs() && net.contains(src) && src_port < net.outputs_in(src) {
                net.connect_output(src, src_port, global_out);
            }
        }
        "disconnect_output" => {
            let arg0 = expr.args.first()?;
            let out = eval_usize(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if out < net.outputs() {
                net.disconnect_output(out);
            }
        }
        "pipe_output" => {
            let arg0 = expr.args.first()?;
            let src = eval_nodeid(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(src) {
                net.pipe_output(src);
            }
        }
        "pass_through" => {
            let arg0 = expr.args.first()?;
            let input = eval_usize(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let output = eval_usize(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if input < net.inputs() && output < net.outputs() {
                net.pass_through(input, output);
            }
        }
        "pipe_all" => {
            let arg0 = expr.args.first()?;
            let src = eval_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let snk = eval_nodeid(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(src) && net.contains(snk) {
                net.pipe_all(src, snk);
            }
        }
        "set_source" => {
            let id = eval_nodeid(expr.args.first()?, lapis)?;
            let chan = eval_usize(expr.args.get(1)?, lapis)?;
            let source = eval_source(expr.args.get(2)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.contains(id) && chan < net.inputs_in(id) {
                if let Source::Local(src_id, src_chan) = source {
                    if id != src_id && net.contains(src_id) && src_chan < net.outputs_in(src_id) {
                        net.set_source(id, chan, source);
                    }
                } else if let Source::Global(g_chan) = source {
                    if g_chan < net.inputs() {
                        net.set_source(id, chan, source);
                    }
                } else {
                    net.set_source(id, chan, source);
                }
            }
        }
        "set_output_source" => {
            let chan = eval_usize(expr.args.first()?, lapis)?;
            let source = eval_source(expr.args.get(1)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if chan < net.outputs() {
                if let Source::Local(src_id, src_chan) = source {
                    if net.contains(src_id) && src_chan < net.outputs_in(src_id) {
                        net.set_output_source(chan, source);
                    }
                } else if let Source::Global(i) = source {
                    if i < net.inputs() {
                        net.set_output_source(chan, source);
                    }
                } else {
                    net.set_output_source(chan, source);
                }
            }
        }
        "commit" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            if net.has_backend() {
                net.commit();
            }
        }
        "set_sample_rate" => {
            let arg = expr.args.first()?;
            let sr = eval_float(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            net.set_sample_rate(sr);
        }
        "reset" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get_mut(&k)?;
            net.reset();
        }
        _ => {}
    }
    None
}

pub fn eval_nodeid(expr: &Expr, lapis: &mut Lapis) -> Option<NodeId> {
    match expr {
        Expr::MethodCall(expr) => method_nodeid(expr, lapis),
        Expr::Path(expr) => path_nodeid(&expr.path, lapis),
        _ => None,
    }
}

fn method_nodeid(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<NodeId> {
    match expr.method.to_string().as_str() {
        "push" => {
            let arg = expr.args.first()?;
            let node = eval_net(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let g = lapis.gmap.get_mut(&k)?;
            Some(g.push(Box::new(node)))
        }
        "chain" => {
            let arg = expr.args.first()?;
            let node = eval_net(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let g = lapis.gmap.get_mut(&k)?;
            Some(g.chain(Box::new(node)))
        }
        "fade_in" => {
            let fade = path_fade(expr.args.first()?)?;
            let fade_time = eval_float_f32(expr.args.get(1)?, lapis)?;
            let unit = Box::new(eval_net(expr.args.get(2)?, lapis)?);
            let k = nth_path_ident(&expr.receiver, 0)?;
            let g = lapis.gmap.get_mut(&k)?;
            Some(g.fade_in(fade, fade_time, unit))
        }
        "nth" => {
            let index = eval_usize(expr.args.first()?, lapis)?;
            if let Expr::MethodCall(ref expr) = *expr.receiver
                && expr.method == "ids"
            {
                let k = nth_path_ident(&expr.receiver, 0)?;
                let g = &lapis.gmap.get(&k)?;
                return g.ids().nth(index).copied();
            }
            None
        }
        _ => None,
    }
}

pub fn eval_path_nodeid(expr: &Expr, lapis: &Lapis) -> Option<NodeId> {
    if let Expr::Path(expr) = expr { path_nodeid(&expr.path, lapis) } else { None }
}

fn path_nodeid(expr: &Path, lapis: &Lapis) -> Option<NodeId> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.idmap.get(&k).copied()
}

macro_rules! tuple_call_match {
    ( $func:ident, $p:expr ) => {{
        match $p.len() {
            1 => Some(Net::wrap(Box::new($func($p[0])))),
            2 => Some(Net::wrap(Box::new($func(($p[0], $p[1]))))),
            3 => Some(Net::wrap(Box::new($func(($p[0], $p[1], $p[2]))))),
            4 => Some(Net::wrap(Box::new($func(($p[0], $p[1], $p[2], $p[3]))))),
            5 => Some(Net::wrap(Box::new($func(($p[0], $p[1], $p[2], $p[3], $p[4]))))),
            6 => Some(Net::wrap(Box::new($func(($p[0], $p[1], $p[2], $p[3], $p[4], $p[5]))))),
            7 => {
                Some(Net::wrap(Box::new($func(($p[0], $p[1], $p[2], $p[3], $p[4], $p[5], $p[6])))))
            }
            8 => Some(Net::wrap(Box::new($func((
                $p[0], $p[1], $p[2], $p[3], $p[4], $p[5], $p[6], $p[7],
            ))))),
            9 => Some(Net::wrap(Box::new($func((
                $p[0], $p[1], $p[2], $p[3], $p[4], $p[5], $p[6], $p[7], $p[8],
            ))))),
            10 => Some(Net::wrap(Box::new($func((
                $p[0], $p[1], $p[2], $p[3], $p[4], $p[5], $p[6], $p[7], $p[8], $p[9],
            ))))),
            _ => None,
        }
    }};
}

fn call_net(expr: &ExprCall, lapis: &mut Lapis) -> Option<Net> {
    let func = nth_path_ident(&expr.func, 0)?;
    let args = accumulate_args(&expr.args, lapis);
    match func.as_str() {
        "Net" => {
            let f = nth_path_ident(&expr.func, 1)?;
            match f.as_str() {
                "new" => {
                    let ins = args.first()?;
                    let outs = args.get(1)?;
                    Some(Net::new(*ins as usize, *outs as usize))
                }
                "scalar" => {
                    let arg0 = expr.args.first()?;
                    let arg1 = expr.args.get(1)?;
                    let chans = eval_usize(arg0, lapis)?;
                    let val = eval_float_f32(arg1, lapis)?;
                    Some(Net::scalar(chans, val))
                }
                _ => None,
            }
        }
        "Box" => {
            if nth_path_ident(&expr.func, 1)? == "new" {
                return eval_net(expr.args.first()?, lapis);
            }
            None
        }
        "add" => {
            let tuple = expr.args.first()?;
            if let Expr::Tuple(expr) = tuple {
                let p = accumulate_args(&expr.elems, lapis);
                tuple_call_match!(add, p)
            } else {
                match args.len() {
                    1 => Some(Net::wrap(Box::new(add(args[0])))),
                    _ => None,
                }
            }
        }
        "adsr_live" => {
            let a = args.first()?;
            let d = args.get(1)?;
            let s = args.get(2)?;
            let r = args.get(3)?;
            Some(Net::wrap(Box::new(adsr_live(*a, *d, *s, *r))))
        }
        "afollow" => {
            let attack = args.first()?;
            let release = args.get(1)?;
            Some(Net::wrap(Box::new(afollow(*attack, *release))))
        }
        "allnest" => {
            let arg = expr.args.first()?;
            let net = eval_net(arg, lapis)?;
            if net.inputs() != 1 || net.outputs() != 1 {
                return None;
            }
            let node = Unit::<U1, U1>::new(Box::new(net));
            Some(Net::wrap(Box::new(allnest(An(node)))))
        }
        "allnest_c" => {
            let coeff = args.first()?;
            let arg = expr.args.get(1)?;
            let net = eval_net(arg, lapis)?;
            if net.inputs() != 1 || net.outputs() != 1 {
                return None;
            }
            let node = Unit::<U1, U1>::new(Box::new(net));
            Some(Net::wrap(Box::new(allnest_c(*coeff, An(node)))))
        }
        "allpass" => Some(Net::wrap(Box::new(allpass()))),
        "allpass_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(allpass_hz(*f, *q))))
        }
        "allpass_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(allpass_q(*q))))
        }
        "allpole" => Some(Net::wrap(Box::new(allpole()))),
        "allpole_delay" => {
            let delay = args.first()?;
            Some(Net::wrap(Box::new(allpole_delay(delay.max(0.0000001)))))
        }
        "bandpass" => Some(Net::wrap(Box::new(bandpass()))),
        "bandpass_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(bandpass_hz(*f, *q))))
        }
        "bandpass_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(bandpass_q(*q))))
        }
        "bandrez" => Some(Net::wrap(Box::new(bandrez()))),
        "bandrez_hz" => {
            let center = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(bandrez_hz(*center, *q))))
        }
        "bandrez_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(bandrez_q(*q))))
        }
        "bassdrum" => {
            let sharpness = args.first()?;
            let pitch0 = args.get(1)?;
            let pitch1 = args.get(2)?;
            Some(Net::wrap(Box::new(bassdrum(*sharpness, *pitch0, *pitch1))))
        }
        "bell" => Some(Net::wrap(Box::new(bell()))),
        "bell_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            let gain = args.get(2)?;
            Some(Net::wrap(Box::new(bell_hz(*f, *q, *gain))))
        }
        "bell_q" => {
            let q = args.first()?;
            let gain = args.get(1)?;
            Some(Net::wrap(Box::new(bell_q(*q, *gain))))
        }
        "biquad" => {
            let a1 = args.first()?;
            let a2 = args.get(1)?;
            let b0 = args.get(2)?;
            let b1 = args.get(3)?;
            let b2 = args.get(4)?;
            Some(Net::wrap(Box::new(biquad(*a1, *a2, *b0, *b1, *b2))))
        }
        "biquad_bank" => None, // TODO
        "branch" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let y = eval_net(arg1, lapis)?;
            if x.inputs() == y.inputs() { Some(x ^ y) } else { None }
        }
        "branchf" | "branchi" => None, //TODO
        "brown" => Some(Net::wrap(Box::new(brown()))),
        "bus" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let y = eval_net(arg1, lapis)?;
            if x.outputs() == y.outputs() && x.inputs() == y.inputs() { Some(x & y) } else { None }
        }
        "busf" | "busi" => None, //TODO
        "butterpass" => Some(Net::wrap(Box::new(butterpass()))),
        "butterpass_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(butterpass_hz(*f))))
        }
        "chorus" => {
            let arg = expr.args.first()?;
            let seed = eval_u64(arg, lapis)?;
            let seperation = args.get(1)?;
            let variation = args.get(2)?;
            let mod_freq = args.get(3)?;
            Some(Net::wrap(Box::new(chorus(seed, *seperation, *variation, *mod_freq))))
        }
        "clip" => Some(Net::wrap(Box::new(clip()))),
        "clip_to" => {
            let min = args.first()?;
            let max = args.get(1)?;
            Some(Net::wrap(Box::new(clip_to(min.min(*max), max.max(*min)))))
        }
        "cymbal" => {
            let seed = eval_i64(expr.args.first()?, lapis)?;
            Some(Net::wrap(Box::new(cymbal(seed))))
        }
        "dbell" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(dbell(shape))))
        }
        "dbell_hz" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            let center = args.first()?;
            let q = args.get(1)?;
            let gain = args.get(2)?;
            Some(Net::wrap(Box::new(dbell_hz(shape, *center, *q, *gain))))
        }
        "dc" | "constant" => {
            let tuple = expr.args.first()?;
            if let Expr::Tuple(expr) = tuple {
                let p = accumulate_args(&expr.elems, lapis);
                tuple_call_match!(dc, p)
            } else {
                match args.len() {
                    1 => Some(Net::wrap(Box::new(dc(args[0])))),
                    _ => None,
                }
            }
        }
        "dcblock" => Some(Net::wrap(Box::new(dcblock()))),
        "dcblock_hz" => {
            let cutoff = args.first()?;
            Some(Net::wrap(Box::new(dcblock_hz(*cutoff))))
        }
        "declick" => Some(Net::wrap(Box::new(declick()))),
        "declick_s" => {
            let t = args.first()?;
            Some(Net::wrap(Box::new(declick_s(*t))))
        }
        "delay" => {
            let t = args.first()?;
            Some(Net::wrap(Box::new(delay(t.max(0.)))))
        }
        "dhighpass" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(dhighpass(shape))))
        }
        "dhighpass_hz" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            let cutoff = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(dhighpass_hz(shape, *cutoff, *q))))
        }
        "dlowpass" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(dlowpass(shape))))
        }
        "dlowpass_hz" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            let cutoff = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(dlowpass_hz(shape, *cutoff, *q))))
        }
        "dresonator" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(dresonator(shape))))
        }
        "dresonator_hz" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            let center = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(dresonator_hz(shape, *center, *q))))
        }
        "dsf_saw" => Some(Net::wrap(Box::new(dsf_saw()))),
        "dsf_saw_r" => {
            let roughness = args.first()?;
            Some(Net::wrap(Box::new(dsf_saw_r(*roughness))))
        }
        "dsf_square" => Some(Net::wrap(Box::new(dsf_square()))),
        "dsf_square_r" => {
            let roughness = args.first()?;
            Some(Net::wrap(Box::new(dsf_square_r(*roughness))))
        }
        "envelope" | "envelope2" | "envelope3" | "envelope_in" => None, //TODO
        "lfo" | "lfo2" | "lfo3" | "lfo_in" => None,                     //TODO
        "fbell" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(fbell(shape))))
        }
        "fbell_hz" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            let center = args.first()?;
            let q = args.get(1)?;
            let gain = args.get(2)?;
            Some(Net::wrap(Box::new(fbell_hz(shape, *center, *q, *gain))))
        }
        "fdn" | "fdn2" => None, //TODO
        "feedback" => {
            let arg = expr.args.get(0)?;
            let net = eval_net(arg, lapis)?;
            if net.inputs() != net.outputs() {
                return None;
            }
            Some(Net::wrap(Box::new(FeedbackUnit::new(0., Box::new(net)))))
        }
        "feedback2" => None, //TODO
        "rfft" => {
            let n = eval_usize(expr.args.first()?, lapis)?;
            let offset = eval_usize(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(An(Rfft::new(n, offset)))))
        }
        "ifft" => {
            let n = eval_usize(expr.args.first()?, lapis)?;
            let offset = eval_usize(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(An(Ifft::new(n, offset)))))
        }
        "fhighpass" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(fhighpass(shape))))
        }
        "fhighpass_hz" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            let cutoff = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(fhighpass_hz(shape, *cutoff, *q))))
        }
        "fir" => {
            let tuple = expr.args.first()?;
            if let Expr::Tuple(expr) = tuple {
                let p = accumulate_args(&expr.elems, lapis);
                tuple_call_match!(fir, p)
            } else {
                match args.len() {
                    1 => Some(Net::wrap(Box::new(fir(args[0])))),
                    _ => None,
                }
            }
        }
        "fir3" => {
            let gain = args.first()?;
            Some(Net::wrap(Box::new(fir3(*gain))))
        }
        "flanger" => {
            let feedback_amount = eval_float_f32(expr.args.first()?, lapis)?;
            let min_delay = eval_float_f32(expr.args.get(1)?, lapis)?;
            let max_delay = eval_float_f32(expr.args.get(2)?, lapis)?;
            let node = (pass() | pass())
                & feedback2(
                    tap(min_delay, max_delay) | zero(),
                    shape(Tanh(feedback_amount)) | pass(),
                );
            let node = node >> (pass() | sink());
            Some(Net::wrap(Box::new(node)))
        }
        "flowpass" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(flowpass(shape))))
        }
        "flowpass_hz" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            let cutoff = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(flowpass_hz(shape, *cutoff, *q))))
        }
        "follow" => {
            let response_time = args.first()?;
            Some(Net::wrap(Box::new(follow(*response_time))))
        }
        "fresonator" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(fresonator(shape))))
        }
        "fresonator_hz" => {
            let arg = expr.args.first()?;
            let shape = call_shape(arg, lapis)?;
            let center = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(fresonator_hz(shape, *center, *q))))
        }
        "hammond" => Some(Net::wrap(Box::new(hammond()))),
        "hammond_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(hammond_hz(*f))))
        }
        "highpass" => Some(Net::wrap(Box::new(highpass()))),
        "highpass_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(highpass_hz(*f, *q))))
        }
        "highpass_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(highpass_q(*q))))
        }
        "highpole" => Some(Net::wrap(Box::new(highpole()))),
        "highpole_hz" => {
            let cutoff = args.first()?;
            Some(Net::wrap(Box::new(highpole_hz(*cutoff))))
        }
        "highshelf" => Some(Net::wrap(Box::new(highshelf()))),
        "highshelf_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            let gain = args.get(2)?;
            Some(Net::wrap(Box::new(highshelf_hz(*f, *q, *gain))))
        }
        "highshelf_q" => {
            let q = args.first()?;
            let gain = args.get(1)?;
            Some(Net::wrap(Box::new(highshelf_q(*q, *gain))))
        }
        "hold" => {
            let variability = args.first()?;
            Some(Net::wrap(Box::new(hold(*variability))))
        }
        "hold_hz" => {
            let f = args.first()?;
            let variability = args.get(1)?;
            Some(Net::wrap(Box::new(hold_hz(*f, *variability))))
        }
        "impulse" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let impulse = Net::wrap(Box::new(impulse::<U1>()));
            let split = Net::wrap(Box::new(MultiSplitUnit::new(1, n)));
            Some(Net::wrap(Box::new(impulse >> split)))
        }
        "input" => {
            let r = lapis.receiver.clone();
            let channels = lapis.in_stream.as_ref()?.0.channels;
            if let (Some(i1), Some(i2)) = (args.first(), args.get(1)) {
                let i1 = *i1 as usize;
                let i2 = *i2 as usize;
                let node = map(move |_: &Frame<f32, U0>| {
                    let mut out = (0., 0.);
                    // receive one frame
                    for _ in 0..channels {
                        if let Ok((channel, s)) = r.try_recv() {
                            if channel == i1 {
                                out.0 = s;
                            }
                            if channel == i2 {
                                out.1 = s;
                            }
                        }
                    }
                    out
                });
                return Some(Net::wrap(Box::new(node)));
            } else if let Some(i) = args.first() {
                let i = *i as usize;
                let node = map(move |_: &Frame<f32, U0>| {
                    let mut out = 0.;
                    for _ in 0..channels {
                        if let Ok((channel, s)) = r.try_recv()
                            && channel == i
                        {
                            out = s;
                        }
                    }
                    out
                });
                return Some(Net::wrap(Box::new(node)));
            }
            None
        }
        "join" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(MultiJoinUnit::new(1, n))))
        }
        "limiter" => {
            let attack = args.first()?;
            let release = args.get(1)?;
            Some(Net::wrap(Box::new(limiter(*attack, *release))))
        }
        "limiter_stereo" => {
            let attack = args.first()?;
            let release = args.get(1)?;
            Some(Net::wrap(Box::new(limiter_stereo(*attack, *release))))
        }
        "lorenz" => Some(Net::wrap(Box::new(lorenz()))),
        "lowpass" => Some(Net::wrap(Box::new(lowpass()))),
        "lowpass_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(lowpass_hz(*f, *q))))
        }
        "lowpass_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(lowpass_q(*q))))
        }
        "lowpole" => Some(Net::wrap(Box::new(lowpole()))),
        "lowpole_hz" => {
            let cutoff = args.first()?;
            Some(Net::wrap(Box::new(lowpole_hz(*cutoff))))
        }
        "lowrez" => Some(Net::wrap(Box::new(lowrez()))),
        "lowrez_hz" => {
            let cutoff = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(lowrez_hz(*cutoff, *q))))
        }
        "lowrez_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(lowrez_q(*q))))
        }
        "lowshelf" => Some(Net::wrap(Box::new(lowshelf()))),
        "lowshelf_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            let gain = args.get(2)?;
            Some(Net::wrap(Box::new(lowshelf_hz(*f, *q, *gain))))
        }
        "lowshelf_q" => {
            let q = args.first()?;
            let gain = args.get(1)?;
            Some(Net::wrap(Box::new(lowshelf_q(*q, *gain))))
        }
        "map" => None, //TODO i'll be seeing you...
        "f" => {
            let f = eval_string(expr.args.first()?, lapis)?;
            match f.as_str() {
                "rise" => Some(Net::wrap(Box::new(maps::rise()))),
                "fall" => Some(Net::wrap(Box::new(maps::fall()))),
                ">" => Some(Net::wrap(Box::new(maps::gt()))),
                "<" => Some(Net::wrap(Box::new(maps::lt()))),
                "==" => Some(Net::wrap(Box::new(maps::eq()))),
                "!=" => Some(Net::wrap(Box::new(maps::neq()))),
                ">=" => Some(Net::wrap(Box::new(maps::ge()))),
                "<=" => Some(Net::wrap(Box::new(maps::le()))),
                "min" => Some(Net::wrap(Box::new(maps::min()))),
                "max" => Some(Net::wrap(Box::new(maps::max()))),
                "pow" => Some(Net::wrap(Box::new(maps::pow()))),
                "rem" => Some(Net::wrap(Box::new(maps::rem()))),
                "rem_euclid" => Some(Net::wrap(Box::new(maps::rem_euclid()))),
                "rem2" => Some(Net::wrap(Box::new(maps::rem2()))),
                "log" => Some(Net::wrap(Box::new(maps::log()))),
                "bitand" => Some(Net::wrap(Box::new(maps::bitand()))),
                "bitor" => Some(Net::wrap(Box::new(maps::bitor()))),
                "bitxor" => Some(Net::wrap(Box::new(maps::bitxor()))),
                "shl" => Some(Net::wrap(Box::new(maps::shl()))),
                "shr" => Some(Net::wrap(Box::new(maps::shr()))),
                "lerp" => Some(Net::wrap(Box::new(maps::lerp()))),
                "lerp11" => Some(Net::wrap(Box::new(maps::lerp11()))),
                "delerp" => Some(Net::wrap(Box::new(maps::delerp()))),
                "delerp11" => Some(Net::wrap(Box::new(maps::delerp11()))),
                "xerp" => Some(Net::wrap(Box::new(maps::xerp()))),
                "xerp11" => Some(Net::wrap(Box::new(maps::xerp11()))),
                "dexerp" => Some(Net::wrap(Box::new(maps::dexerp()))),
                "dexerp11" => Some(Net::wrap(Box::new(maps::dexerp11()))),
                "abs" => Some(Net::wrap(Box::new(maps::abs()))),
                "signum" => Some(Net::wrap(Box::new(maps::signum()))),
                "floor" => Some(Net::wrap(Box::new(maps::floor()))),
                "fract" => Some(Net::wrap(Box::new(maps::fract()))),
                "ceil" => Some(Net::wrap(Box::new(maps::ceil()))),
                "round" => Some(Net::wrap(Box::new(maps::round()))),
                "sqrt" => Some(Net::wrap(Box::new(maps::sqrt()))),
                "exp" => Some(Net::wrap(Box::new(maps::exp()))),
                "exp2" => Some(Net::wrap(Box::new(maps::exp2()))),
                "exp10" => Some(Net::wrap(Box::new(maps::exp10()))),
                "exp_m1" => Some(Net::wrap(Box::new(maps::exp_m1()))),
                "ln_1p" => Some(Net::wrap(Box::new(maps::ln_1p()))),
                "ln" => Some(Net::wrap(Box::new(maps::ln()))),
                "log2" => Some(Net::wrap(Box::new(maps::log2()))),
                "log10" => Some(Net::wrap(Box::new(maps::log10()))),
                "hypot" => Some(Net::wrap(Box::new(maps::hypot()))),
                "atan2" => Some(Net::wrap(Box::new(maps::atan2()))),
                "to_pol" => Some(Net::wrap(Box::new(maps::to_pol()))),
                "to_car" => Some(Net::wrap(Box::new(maps::to_car()))),
                "to_deg" => Some(Net::wrap(Box::new(maps::to_deg()))),
                "to_rad" => Some(Net::wrap(Box::new(maps::to_rad()))),
                "sin" => Some(Net::wrap(Box::new(maps::sin()))),
                "cos" => Some(Net::wrap(Box::new(maps::cos()))),
                "tan" => Some(Net::wrap(Box::new(maps::tan()))),
                "asin" => Some(Net::wrap(Box::new(maps::asin()))),
                "acos" => Some(Net::wrap(Box::new(maps::acos()))),
                "atan" => Some(Net::wrap(Box::new(maps::atan()))),
                "sinh" => Some(Net::wrap(Box::new(maps::sinh()))),
                "cosh" => Some(Net::wrap(Box::new(maps::cosh()))),
                "tanh" => Some(Net::wrap(Box::new(maps::tanh()))),
                "asinh" => Some(Net::wrap(Box::new(maps::asinh()))),
                "acosh" => Some(Net::wrap(Box::new(maps::acosh()))),
                "atanh" => Some(Net::wrap(Box::new(maps::atanh()))),
                "squared" => Some(Net::wrap(Box::new(maps::squared()))),
                "cubed" => Some(Net::wrap(Box::new(maps::cubed()))),
                "dissonance" => Some(Net::wrap(Box::new(maps::dissonance()))),
                "dissonance_max" => Some(Net::wrap(Box::new(maps::dissonance_max()))),
                "db_amp" => Some(Net::wrap(Box::new(maps::db_amp()))),
                "amp_db" => Some(Net::wrap(Box::new(maps::amp_db()))),
                "a_weight" => Some(Net::wrap(Box::new(maps::a_weight()))),
                "m_weight" => Some(Net::wrap(Box::new(maps::m_weight()))),
                "spline" => Some(Net::wrap(Box::new(maps::spline()))),
                "spline_mono" => Some(Net::wrap(Box::new(maps::spline_mono()))),
                "softsign" => Some(Net::wrap(Box::new(maps::softsign()))),
                "softexp" => Some(Net::wrap(Box::new(maps::softexp()))),
                "softmix" => Some(Net::wrap(Box::new(maps::softmix()))),
                "smooth3" => Some(Net::wrap(Box::new(maps::smooth3()))),
                "smooth5" => Some(Net::wrap(Box::new(maps::smooth5()))),
                "smooth7" => Some(Net::wrap(Box::new(maps::smooth7()))),
                "smooth9" => Some(Net::wrap(Box::new(maps::smooth9()))),
                "uparc" => Some(Net::wrap(Box::new(maps::uparc()))),
                "downarc" => Some(Net::wrap(Box::new(maps::downarc()))),
                "sine_ease" => Some(Net::wrap(Box::new(maps::sine_ease()))),
                "sin_hz" => Some(Net::wrap(Box::new(maps::sin_hz()))),
                "cos_hz" => Some(Net::wrap(Box::new(maps::cos_hz()))),
                "sqr_hz" => Some(Net::wrap(Box::new(maps::sqr_hz()))),
                "tri_hz" => Some(Net::wrap(Box::new(maps::tri_hz()))),
                "rnd1" => Some(Net::wrap(Box::new(maps::rnd1()))),
                "rnd2" => Some(Net::wrap(Box::new(maps::rnd2()))),
                "spline_noise" => Some(Net::wrap(Box::new(maps::spline_noise()))),
                "fractal_noise" => Some(Net::wrap(Box::new(maps::fractal_noise()))),
                "recip" => Some(Net::wrap(Box::new(maps::recip()))),
                "normal" => Some(Net::wrap(Box::new(maps::normal()))),
                "wrap" => Some(Net::wrap(Box::new(maps::wrap()))),
                "mirror" => Some(Net::wrap(Box::new(maps::mirror()))),
                _ => None,
            }
        }
        "meter" => {
            let arg = expr.args.first()?;
            let m = eval_meter(arg, lapis)?;
            Some(Net::wrap(Box::new(meter(m))))
        }
        "mls" => Some(Net::wrap(Box::new(mls()))),
        "mls_bits" => {
            let arg = expr.args.first()?;
            let n = eval_u64(arg, lapis)?;
            Some(Net::wrap(Box::new(mls_bits(n))))
        }
        "monitor" => {
            let arg0 = expr.args.first()?;
            let shared = eval_shared(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let meter = eval_meter(arg1, lapis)?;
            Some(Net::wrap(Box::new(monitor(&shared, meter))))
        }
        "moog" => Some(Net::wrap(Box::new(moog()))),
        "moog_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(moog_hz(*f, *q))))
        }
        "moog_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(moog_q(*q))))
        }
        "morph" => Some(Net::wrap(Box::new(morph()))),
        "morph_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            let morph = args.get(2)?;
            Some(Net::wrap(Box::new(lowshelf_hz(*f, *q, *morph))))
        }
        "mul" => {
            let tuple = expr.args.first()?;
            if let Expr::Tuple(expr) = tuple {
                let p = accumulate_args(&expr.elems, lapis);
                tuple_call_match!(mul, p)
            } else {
                match args.len() {
                    1 => Some(Net::wrap(Box::new(mul(args[0])))),
                    _ => None,
                }
            }
        }
        "multijoin" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let m = nth_path_generic(&expr.func, 1)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(MultiJoinUnit::new(n, m))))
        }
        "multipass" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let mut g = Net::new(0, 0);
            for _ in 0..n {
                g = g | pass();
            }
            Some(Net::wrap(Box::new(g)))
        }
        "multisink" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let mut g = Net::new(0, 0);
            for _ in 0..n {
                g = g | sink();
            }
            Some(Net::wrap(Box::new(g)))
        }
        "multisplit" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let m = nth_path_generic(&expr.func, 1)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(MultiSplitUnit::new(n, m))))
        }
        "multitap" | "multitap_linear" => None, //TODO
        "multitick" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let mut g = Net::new(0, 0);
            for _ in 0..n {
                g = g | tick();
            }
            Some(Net::wrap(Box::new(g)))
        }
        "multizero" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let mut g = Net::new(0, 0);
            for _ in 0..n {
                g = g | zero();
            }
            Some(Net::wrap(Box::new(g)))
        }
        "noise" => Some(Net::wrap(Box::new(noise()))),
        "notch" => Some(Net::wrap(Box::new(notch()))),
        "notch_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(notch_hz(*f, *q))))
        }
        "notch_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(notch_q(*q))))
        }
        "organ" => Some(Net::wrap(Box::new(organ()))),
        "organ_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(organ_hz(*f))))
        }
        "oversample" => None, //TODO
        "pan" => {
            let p = args.first()?;
            Some(Net::wrap(Box::new(pan(*p))))
        }
        "panner" => Some(Net::wrap(Box::new(panner()))),
        "pass" => Some(Net::wrap(Box::new(pass()))),
        "peak" => Some(Net::wrap(Box::new(peak()))),
        "peak_hz" => {
            let f = args.first()?;
            let q = args.get(1)?;
            Some(Net::wrap(Box::new(peak_hz(*f, *q))))
        }
        "peak_q" => {
            let q = args.first()?;
            Some(Net::wrap(Box::new(peak_q(*q))))
        }
        "pebbles" => {
            let speed = eval_float_f32(expr.args.first()?, lapis)?;
            let seed = eval_u64(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(pebbles(speed, seed))))
        }
        "phaser" => {
            let feedback_amount = eval_float_f32(expr.args.first()?, lapis)?;
            let node = (pass() | pass())
                & feedback(
                    pipei::<U10, _, _>(|_i| add((0.0, 0.1)) >> !allpole())
                        >> (mul(feedback_amount) | sink() | zero()),
                );
            let node = (pass() | map(|i: &Frame<f32, U1>| lerp(2.0, 20.0, clamp01(i[0]))))
                >> node
                >> (pass() | sink());
            Some(Net::wrap(Box::new(node)))
        }
        "pink" => Some(Net::wrap(Box::new(pink()))),
        "pinkpass" => Some(Net::wrap(Box::new(pinkpass()))),
        "pipe" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let y = eval_net(arg1, lapis)?;
            if x.outputs() == y.inputs() { Some(x >> y) } else { None }
        }
        "pipef" | "pipei" => None, //TODO
        "pluck" => {
            let freq = args.first()?;
            let gain_per_sec = args.get(1)?;
            let hf_damp = args.get(2)?;
            Some(Net::wrap(Box::new(pluck(*freq, *gain_per_sec, *hf_damp))))
        }
        "poly_saw" => Some(Net::wrap(Box::new(poly_saw()))),
        "poly_saw_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(poly_saw_hz(*f))))
        }
        "poly_square" => Some(Net::wrap(Box::new(poly_square()))),
        "poly_square_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(poly_square_hz(*f))))
        }
        "product" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let y = eval_net(arg1, lapis)?;
            if x.outputs() == y.outputs() { Some(x * y) } else { None }
        }
        "pulse" => Some(Net::wrap(Box::new(pulse()))),
        "ramp" => Some(Net::wrap(Box::new(ramp()))),
        "ramp_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(ramp_hz(*f))))
        }
        "resample" => None, //TODO
        "resonator" => Some(Net::wrap(Box::new(resonator()))),
        "resonator_hz" => {
            let center = args.first()?;
            let bandwidth = args.get(1)?;
            Some(Net::wrap(Box::new(resonator_hz(*center, *bandwidth))))
        }
        "resynth" => None, //TODO
        "reverb2_stereo" => {
            let room = args.first()?;
            let time = args.get(1)?;
            let diffusion = args.get(2)?;
            let modulation = args.get(3)?;
            let arg = expr.args.get(4)?;
            let net = eval_net(arg, lapis)?;
            if net.inputs() != 1 || net.outputs() != 1 {
                return None;
            }
            let node = An(Unit::<U1, U1>::new(Box::new(net)));
            Some(Net::wrap(Box::new(reverb2_stereo(*room, *time, *diffusion, *modulation, node))))
        }
        "reverb3_stereo" => {
            let time = args.first()?;
            let diffusion = args.get(1)?;
            let arg = expr.args.get(2)?;
            let net = eval_net(arg, lapis)?;
            if net.inputs() != 1 || net.outputs() != 1 {
                return None;
            }
            let node = An(Unit::<U1, U1>::new(Box::new(net)));
            Some(Net::wrap(Box::new(reverb3_stereo(*time, *diffusion, node))))
        }
        "reverb4_stereo" => {
            let room = args.first()?;
            let time = args.get(1)?;
            Some(Net::wrap(Box::new(reverb4_stereo(*room, *time))))
        }
        "reverb4_stereo_delays" => {
            let arg = expr.args.first()?;
            let mut delays = eval_vec(arg, lapis)?;
            let time = args.first()?;
            if delays.len() != 32 {
                return None;
            }
            for d in delays.iter_mut() {
                *d = d.max(0.);
            }
            Some(Net::wrap(Box::new(reverb4_stereo_delays(&delays, *time))))
        }
        "reverb_stereo" => {
            let room = args.first()?;
            let time = args.get(1)?;
            let damp = args.get(2)?;
            Some(Net::wrap(Box::new(reverb_stereo(*room, *time, *damp))))
        }
        "reverse" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(ReverseUnit::new(n))))
        }
        "risset_glissando" => {
            let up = eval_bool(expr.args.first()?, lapis)?;
            Some(Net::wrap(Box::new(risset_glissando(up))))
        }
        "rossler" => Some(Net::wrap(Box::new(rossler()))),
        "rotate" => {
            let angle = args.first()?;
            let gain = args.get(1)?;
            Some(Net::wrap(Box::new(rotate(*angle, *gain))))
        }
        "saw" => Some(Net::wrap(Box::new(saw()))),
        "saw_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(saw_hz(*f))))
        }
        "shape" => {
            let arg = expr.args.first()?;
            let shp = call_shape(arg, lapis)?;
            Some(Net::wrap(Box::new(shape(shp))))
        }
        "shape_fn" => None, //TODO
        "sine" => Some(Net::wrap(Box::new(sine()))),
        "sine_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(sine_hz(*f))))
        }
        "sink" => Some(Net::wrap(Box::new(sink()))),
        "snaredrum" => {
            let seed = eval_i64(expr.args.first()?, lapis)?;
            let sharpness = eval_float_f32(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(snaredrum(seed, sharpness))))
        }
        "snoop" => None, // TODO you shouldn't be here..
        "soft_saw" => Some(Net::wrap(Box::new(soft_saw()))),
        "soft_saw_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(soft_saw_hz(*f))))
        }
        "split" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(MultiSplitUnit::new(1, n))))
        }
        "square" => Some(Net::wrap(Box::new(square()))),
        "square_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(square_hz(*f))))
        }
        "stack" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let y = eval_net(arg1, lapis)?;
            Some(x | y)
        }
        "stackf" | "stacki" => None, //TODO
        "sub" => {
            let tuple = expr.args.first()?;
            if let Expr::Tuple(expr) = tuple {
                let p = accumulate_args(&expr.elems, lapis);
                tuple_call_match!(sub, p)
            } else {
                match args.len() {
                    1 => Some(Net::wrap(Box::new(sub(args[0])))),
                    _ => None,
                }
            }
        }
        "sum" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let y = eval_net(arg1, lapis)?;
            if x.outputs() == y.outputs() { Some(x + y) } else { None }
        }
        "sumf" | "sumi" => None, //TODO
        "t" => Some(Net::wrap(Box::new(lfo(|t| t)))),
        "tap" => {
            let min = args.first()?;
            let max = args.get(1)?;
            Some(Net::wrap(Box::new(tap(*min, *max))))
        }
        "tap_linear" => {
            let min = args.first()?;
            let max = args.get(1)?;
            Some(Net::wrap(Box::new(tap_linear(*min, *max))))
        }
        "thru" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            Some(!x)
        }
        "tick" => Some(Net::wrap(Box::new(tick()))),
        "timer" => {
            let arg = expr.args.first()?;
            let shared = eval_shared(arg, lapis)?;
            Some(Net::wrap(Box::new(timer(&shared))))
        }
        "triangle" => Some(Net::wrap(Box::new(triangle()))),
        "triangle_hz" => {
            let f = args.first()?;
            Some(Net::wrap(Box::new(triangle_hz(*f))))
        }
        "unit" => None,   //TODO return the input net?
        "update" => None, //TODO
        "var" => {
            let arg = expr.args.first()?;
            let shared = eval_shared(arg, lapis)?;
            Some(Net::wrap(Box::new(var(&shared))))
        }
        "var_fn" => None, // TODO
        "wavech" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2);
            let k = nth_path_ident(arg0, 0)?;
            let wave = lapis.wmap.get(&k)?.clone();
            let chan = eval_usize(arg1, lapis)?;
            if chan < wave.channels() {
                let loop_point = if let Some(arg) = arg2 { eval_usize(arg, lapis) } else { None };
                Some(Net::wrap(Box::new(wavech(&wave, chan, loop_point))))
            } else {
                None
            }
        }
        "wavech_at" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let arg3 = expr.args.get(3)?;
            let arg4 = expr.args.get(4);
            let k = nth_path_ident(arg0, 0)?;
            let wave = lapis.wmap.get(&k)?.clone();
            let chan = eval_usize(arg1, lapis)?;
            let start = eval_usize(arg2, lapis)?;
            let end = eval_usize(arg3, lapis)?;
            if chan < wave.channels() && end <= wave.len() {
                let loop_point = if let Some(arg) = arg4 { eval_usize(arg, lapis) } else { None };
                Some(Net::wrap(Box::new(wavech_at(&wave, chan, start, end, loop_point))))
            } else {
                None
            }
        }
        "white" => Some(Net::wrap(Box::new(white()))),
        "zero" => Some(Net::wrap(Box::new(zero()))),
        "select" => {
            let mut units: Vec<Box<dyn AudioUnit>> = Vec::new();
            for arg in &expr.args {
                if let Some(unit) = eval_net(arg, lapis)
                    && unit.inputs() == 0
                    && unit.outputs() == 1
                {
                    units.push(Box::new(unit));
                }
            }
            Some(Net::wrap(Box::new(An(Select::new(units)))))
        }
        "fade_select" => {
            let mut units: Vec<Box<dyn AudioUnit>> = Vec::new();
            for arg in &expr.args {
                if let Some(unit) = eval_net(arg, lapis)
                    && unit.inputs() == 0
                    && unit.outputs() == 1
                {
                    units.push(Box::new(unit));
                }
            }
            Some(Net::wrap(Box::new(An(FadeSelect::new(units)))))
        }
        "seq" => {
            let mut units: Vec<Box<dyn AudioUnit>> = Vec::new();
            for arg in &expr.args {
                if let Some(unit) = eval_net(arg, lapis)
                    && unit.inputs() == 0
                    && unit.outputs() == 1
                {
                    units.push(Box::new(unit));
                }
            }
            Some(Net::wrap(Box::new(An(Seq::new(units)))))
        }
        "shift_reg" => Some(Net::wrap(Box::new(An(ShiftReg::new())))),
        "quantizer" => {
            let vec = eval_vec(expr.args.first()?, lapis)?;
            let range = vec.last()? - vec.first()?;
            Some(Net::wrap(Box::new(An(Quantizer::new(vec, range)))))
        }
        "kr" => {
            let unit = Box::new(eval_net(expr.args.first()?, lapis)?);
            let n = eval_usize(expr.args.get(1)?, lapis)?;
            let preserve_time = eval_bool(expr.args.get(2)?, lapis)?;
            Some(Net::wrap(Box::new(Kr::new(unit, n, preserve_time))))
        }
        "reset" => {
            let unit = Box::new(eval_net(expr.args.first()?, lapis)?);
            if unit.inputs() != 0 || unit.outputs() != 1 {
                return None;
            }
            let dur = eval_float_f32(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(An(Reset::new(unit, dur)))))
        }
        "trig_reset" => {
            let unit = Box::new(eval_net(expr.args.first()?, lapis)?);
            if unit.inputs() != 0 || unit.outputs() != 1 {
                return None;
            }
            Some(Net::wrap(Box::new(An(TrigReset::new(unit)))))
        }
        "reset_v" => {
            let unit = Box::new(eval_net(expr.args.first()?, lapis)?);
            if unit.inputs() != 0 || unit.outputs() != 1 {
                return None;
            }
            Some(Net::wrap(Box::new(An(ResetV::new(unit)))))
        }
        "snh" => Some(Net::wrap(Box::new(An(SnH::new())))),
        "euclid" => Some(Net::wrap(Box::new(An(EuclidSeq::new())))),
        "resample1" => {
            let unit = Box::new(eval_net(expr.args.first()?, lapis)?);
            if unit.inputs() != 0 || unit.outputs() != 1 {
                return None;
            }
            let node = Unit::<U0, U1>::new(unit);
            Some(Net::wrap(Box::new(resample(An(node)))))
        }
        "bitcrush" => Some(Net::wrap(Box::new(maps::bitcrush()))),
        "gate" => Some(Net::wrap(Box::new(An(Gate::new(*args.first()? as f64))))),
        "phase_synth" => {
            let table = eval_string(expr.args.first()?, lapis)?;
            let table = match table.as_str() {
                "hammond" => hammond_table(),
                "organ" => organ_table(),
                "saw" => saw_table(),
                "soft_saw" => soft_saw_table(),
                "square" => square_table(),
                "triangle" => triangle_table(),
                "sine" => sine_table(),
                _ => return None,
            };
            Some(Net::wrap(Box::new(An(PhaseSynth::new(table)))))
        }
        "unsteady" => {
            let times = eval_vec(expr.args.first()?, lapis)?;
            let looping = eval_bool(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(An(Unsteady::new(times, looping)))))
        }
        "unsteady_no_reset" | "unsteady_nr" => {
            let times = eval_vec(expr.args.first()?, lapis)?;
            let looping = eval_bool(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(An(Unsteady::new(times, looping).no_reset()))))
        }
        "unsteady_ramp" => {
            let times = eval_vec(expr.args.first()?, lapis)?;
            let looping = eval_bool(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(An(UnsteadyRamp::new(times, looping)))))
        }
        "atomic_synth" => {
            let k = nth_path_ident(expr.args.first()?, 0)?;
            if let Some(table) = lapis.atomic_table_map.get(&k) {
                let mut synth = AtomicSynth::<f32>::new(table.clone());
                if let Some(arg1) = expr.args.get(1)
                    && let Some(interp) = eval_string(arg1, lapis)
                {
                    if interp == "linear" {
                        synth.set_interpolation(Interpolation::Linear);
                    } else if interp == "cubic" {
                        synth.set_interpolation(Interpolation::Cubic);
                    }
                }
                return Some(Net::wrap(Box::new(An(synth))));
            }
            None
        }
        "ahr" => {
            let a = eval_float_f32(expr.args.first()?, lapis)?;
            let h = eval_float_f32(expr.args.get(1)?, lapis)?;
            let r = eval_float_f32(expr.args.get(2)?, lapis)?;
            Some(Net::wrap(Box::new(ahr(a, h, r))))
        }
        "step" => {
            let mut units: Vec<Box<dyn AudioUnit>> = Vec::new();
            for arg in &expr.args {
                if let Some(unit) = eval_net(arg, lapis)
                    && unit.inputs() == 0
                    && unit.outputs() == 1
                {
                    units.push(Box::new(unit));
                }
            }
            Some(Net::wrap(Box::new(An(Step::new(units)))))
        }
        "filter_step" => {
            let mut units: Vec<Box<dyn AudioUnit>> = Vec::new();
            for arg in &expr.args {
                if let Some(unit) = eval_net(arg, lapis)
                    && unit.inputs() == 1
                    && unit.outputs() == 1
                {
                    units.push(Box::new(unit));
                }
            }
            Some(Net::wrap(Box::new(An(FilterStep::new(units)))))
        }
        "atomic_phase" => {
            let k = nth_path_ident(expr.args.first()?, 0)?;
            let table = lapis.atomic_table_map.get(&k)?;
            let mut interp = Interpolation::Nearest;
            if let Some(arg1) = expr.args.get(1)
                && let Some(i) = eval_string(arg1, lapis)
            {
                if i == "linear" {
                    interp = Interpolation::Linear;
                } else if i == "cubic" {
                    interp = Interpolation::Cubic;
                }
            }
            Some(Net::wrap(Box::new(maps::atomic_phase(table.clone(), interp))))
        }
        _ => None,
    }
}
