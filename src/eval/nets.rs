use crate::eval::*;
use fundsp::sound::*;
use std::num::Wrapping;

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
            if let Some(seq) = &mut lapis.seqmap.get_mut(&k) {
                if !seq.has_backend() {
                    return Some(Net::wrap(Box::new(seq.backend())));
                }
            } else if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                if !g.has_backend() {
                    return Some(Net::wrap(Box::new(g.backend())));
                }
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
            let net = &mut lapis.gmap.get_mut(&k)?;
            if net.contains(id) {
                Some(Net::wrap(net.remove(id)))
            } else {
                None
            }
        }
        "remove_link" => {
            let arg = expr.args.first()?;
            let id = eval_nodeid(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
            if net.contains(id)
                && unit.inputs() == net.inputs_in(id)
                && unit.outputs() == net.outputs_in(id)
            {
                return Some(Net::wrap(net.replace(id, Box::new(unit))));
            }
            None
        }
        "phase" => {
            let p = eval_float(expr.args.first()?, lapis)?;
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
    let left_float = eval_float(&expr.left, lapis);
    let right_float = eval_float(&expr.right, lapis);
    if left_net.is_some() && right_net.is_some() {
        let (left, right) = (left_net.unwrap(), right_net.unwrap());
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
    } else if let (Some(left), Some(right)) = (left_net, right_float) {
        match expr.op {
            BinOp::Sub(_) => Some(left - right),
            BinOp::Mul(_) => Some(left * right),
            BinOp::Add(_) => Some(left + right),
            _ => None,
        }
    } else if let (Some(left), Some(right)) = (left_float, right_net) {
        match expr.op {
            BinOp::Sub(_) => Some(left - right),
            BinOp::Mul(_) => Some(left * right),
            BinOp::Add(_) => Some(left + right),
            _ => None,
        }
    } else {
        None
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
            let net = &mut lapis.gmap.get_mut(&k)?;
            if net.contains(id) {
                net.remove(id);
            }
        }
        "remove_link" => {
            let arg = expr.args.first()?;
            let id = eval_nodeid(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let time = eval_float(arg2, lapis)?;
            let arg3 = expr.args.get(3)?;
            let unit = eval_net(arg3, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
            if global_in < net.inputs() && net.contains(snk) && snk_port < net.inputs_in(snk) {
                net.connect_input(global_in, snk, snk_port);
            }
        }
        "pipe_input" => {
            let arg0 = expr.args.first()?;
            let snk = eval_nodeid(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
            if global_out < net.outputs() && net.contains(src) && src_port < net.outputs_in(src) {
                net.connect_output(src, src_port, global_out);
            }
        }
        "disconnect_output" => {
            let arg0 = expr.args.first()?;
            let out = eval_usize(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            if out < net.outputs() {
                net.disconnect_output(out);
            }
        }
        "pipe_output" => {
            let arg0 = expr.args.first()?;
            let src = eval_nodeid(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
            if net.contains(src) && net.contains(snk) {
                net.pipe_all(src, snk);
            }
        }
        "set_source" => {
            let id = eval_nodeid(expr.args.first()?, lapis)?;
            let chan = eval_usize(expr.args.get(1)?, lapis)?;
            let source = eval_source(expr.args.get(2)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let net = &mut lapis.gmap.get_mut(&k)?;
            if net.has_backend() {
                net.commit();
            }
        }
        "set_sample_rate" => {
            let arg = expr.args.first()?;
            let sr = eval_float(arg, lapis)? as f64;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.set_sample_rate(sr);
        }
        "reset" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
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
            let g = &mut lapis.gmap.get_mut(&k)?;
            Some(g.push(Box::new(node)))
        }
        "chain" => {
            let arg = expr.args.first()?;
            let node = eval_net(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let g = &mut lapis.gmap.get_mut(&k)?;
            Some(g.chain(Box::new(node)))
        }
        "fade_in" => {
            let fade = path_fade(expr.args.first()?)?;
            let fade_time = eval_float(expr.args.get(1)?, lapis)?;
            let unit = Box::new(eval_net(expr.args.get(2)?, lapis)?);
            let k = nth_path_ident(&expr.receiver, 0)?;
            let g = &mut lapis.gmap.get_mut(&k)?;
            Some(g.fade_in(fade, fade_time, unit))
        }
        "nth" => {
            let index = eval_usize(expr.args.first()?, lapis)?;
            if let Expr::MethodCall(ref expr) = *expr.receiver {
                if expr.method == "ids" {
                    let k = nth_path_ident(&expr.receiver, 0)?;
                    let g = &lapis.gmap.get(&k)?;
                    return g.ids().nth(index).copied();
                }
            }
            None
        }
        _ => None,
    }
}

pub fn eval_path_nodeid(expr: &Expr, lapis: &Lapis) -> Option<NodeId> {
    if let Expr::Path(expr) = expr {
        path_nodeid(&expr.path, lapis)
    } else {
        None
    }
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
                    let val = eval_float(arg1, lapis)?;
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
            Some(Net::wrap(Box::new(allpole_delay(*delay))))
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
            if x.inputs() == y.inputs() {
                Some(x ^ y)
            } else {
                None
            }
        }
        "branchf" | "branchi" => None, //TODO
        "brown" => Some(Net::wrap(Box::new(brown()))),
        "bus" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let y = eval_net(arg1, lapis)?;
            if x.outputs() == y.outputs() && x.inputs() == y.inputs() {
                Some(x & y)
            } else {
                None
            }
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
            Some(Net::wrap(Box::new(clip_to(*min, *max))))
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
            Some(Net::wrap(Box::new(delay(*t))))
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
        "flanger" => None, //TODO
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
            #[cfg(feature = "gui")]
            {
                let (lr, rr) = &lapis.receivers;
                Some(Net::wrap(Box::new(An(InputNode::new(lr.clone(), rr.clone())))))
            }
            #[cfg(not(feature = "gui"))]
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
            let arg = expr.args.first()?;
            let mut f = String::new();
            if let Expr::Lit(expr) = arg {
                if let Lit::Str(expr) = &expr.lit {
                    f = expr.value();
                }
            }
            match f.as_str() {
                "" => None,
                "rise" => Some(Net::wrap(Box::new(
                    (pass() ^ tick())
                        >> map(|i: &Frame<f32, U2>| if i[0] > i[1] { 1. } else { 0. }),
                ))),
                "fall" => Some(Net::wrap(Box::new(
                    (pass() ^ tick())
                        >> map(|i: &Frame<f32, U2>| if i[0] < i[1] { 1. } else { 0. }),
                ))),
                ">" => Some(Net::wrap(Box::new(map(
                    |i: &Frame<f32, U2>| if i[0] > i[1] { 1. } else { 0. },
                )))),
                "<" => Some(Net::wrap(Box::new(map(
                    |i: &Frame<f32, U2>| if i[0] < i[1] { 1. } else { 0. },
                )))),
                "==" => Some(Net::wrap(Box::new(map(
                    |i: &Frame<f32, U2>| if i[0] == i[1] { 1. } else { 0. },
                )))),
                "!=" => Some(Net::wrap(Box::new(map(
                    |i: &Frame<f32, U2>| if i[0] != i[1] { 1. } else { 0. },
                )))),
                ">=" => Some(Net::wrap(Box::new(map(
                    |i: &Frame<f32, U2>| if i[0] >= i[1] { 1. } else { 0. },
                )))),
                "<=" => Some(Net::wrap(Box::new(map(
                    |i: &Frame<f32, U2>| if i[0] <= i[1] { 1. } else { 0. },
                )))),
                "min" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].min(i[1]))))),
                "max" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].max(i[1]))))),
                "pow" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].pow(i[1]))))),
                "mod" | "rem" | "rem_euclid" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].rem_euclid(i[1])))))
                }
                "log" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].log(i[1]))))),
                "bitand" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| {
                    (i[0] as i32 & i[1] as i32) as f32
                })))),
                "bitor" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| {
                    (i[0] as i32 | i[1] as i32) as f32
                })))),
                "bitxor" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| {
                    (i[0] as i32 ^ i[1] as i32) as f32
                })))),
                "shl" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| {
                    let i = Wrapping(i[0] as i32) << (i[1] as usize);
                    i.0 as f32
                })))),
                "shr" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| {
                    let i = Wrapping(i[0] as i32) >> (i[1] as usize);
                    i.0 as f32
                })))),
                "lerp" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| lerp(i[0], i[1], i[2])))))
                }
                "lerp11" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| lerp11(i[0], i[1], i[2])))))
                }
                "delerp" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| delerp(i[0], i[1], i[2])))))
                }
                "delerp11" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| delerp11(i[0], i[1], i[2])))))
                }
                "xerp" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| xerp(i[0], i[1], i[2])))))
                }
                "xerp11" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| xerp11(i[0], i[1], i[2])))))
                }
                "dexerp" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| dexerp(i[0], i[1], i[2])))))
                }
                "dexerp11" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| dexerp11(i[0], i[1], i[2])))))
                }
                "abs" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].abs())))),
                "signum" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].signum())))),
                "floor" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].floor())))),
                "fract" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].fract())))),
                "ceil" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].ceil())))),
                "round" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].round())))),
                "sqrt" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sqrt())))),
                "exp" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].exp())))),
                "exp2" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].exp2())))),
                "exp10" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| (exp10(i[0])))))),
                "exp_m1" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| (i[0].ln_1p()))))),
                "ln_1p" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| (i[0].exp_m1()))))),
                "ln" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].ln())))),
                "log2" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].log2())))),
                "log10" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].log10())))),
                "hypot" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].hypot(i[1]))))),
                "atan2" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].atan2(i[1]))))),
                "sin" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sin())))),
                "cos" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].cos())))),
                "tan" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].tan())))),
                "asin" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].asin())))),
                "acos" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].acos())))),
                "atan" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].atan())))),
                "sinh" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sinh())))),
                "cosh" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].cosh())))),
                "tanh" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].tanh())))),
                "asinh" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].asinh())))),
                "acosh" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].acosh())))),
                "atanh" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].atanh())))),
                "squared" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0] * i[0])))),
                "cubed" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0] * i[0] * i[0])))),
                "dissonance" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| dissonance(i[0], i[1])))))
                }
                "dissonance_max" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| dissonance_max(i[0])))))
                }
                "db_amp" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| db_amp(i[0]))))),
                "amp_db" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| amp_db(i[0]))))),
                "a_weight" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| a_weight(i[0]))))),
                "m_weight" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| m_weight(i[0]))))),
                "spline" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U5>| {
                    spline(i[0], i[1], i[2], i[3], i[4])
                })))),
                "spline_mono" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U5>| {
                    spline_mono(i[0], i[1], i[2], i[3], i[4])
                })))),
                "softsign" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| softsign(i[0]))))),
                "softexp" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| softexp(i[0]))))),
                "softmix" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U3>| softmix(i[0], i[1], i[2])))))
                }
                "smooth3" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth3(i[0]))))),
                "smooth5" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth5(i[0]))))),
                "smooth7" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth7(i[0]))))),
                "smooth9" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth9(i[0]))))),
                "uparc" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| uparc(i[0]))))),
                "downarc" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| downarc(i[0]))))),
                "sine_ease" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| sine_ease(i[0]))))),
                "sin_hz" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| sin_hz(i[0], i[1]))))),
                "cos_hz" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| cos_hz(i[0], i[1]))))),
                "sqr_hz" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| sqr_hz(i[0], i[1]))))),
                "tri_hz" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| tri_hz(i[0], i[1]))))),
                "semitone_ratio" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| semitone_ratio(i[0])))))
                }
                "rnd1" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| rnd1(i[0] as u64) as f32))))
                }
                "rnd2" => {
                    Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| rnd2(i[0] as u64) as f32))))
                }
                "spline_noise" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| {
                    spline_noise(i[0] as u64, i[1]) as f32
                })))),
                "fractal_noise" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U4>| {
                    fractal_noise(i[0] as u64, i[1].min(1.) as i64, i[2], i[3]) as f32
                })))),
                "pol" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| {
                    (i[0].hypot(i[1]), i[1].atan2(i[0]))
                })))),
                "car" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U2>| {
                    (i[0] * i[1].cos(), i[0] * i[1].sin())
                })))),
                "deg" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].to_degrees())))),
                "rad" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].to_radians())))),
                "recip" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].recip())))),
                "normal" => {
                    Some(Net::wrap(Box::new(map(
                        |i: &Frame<f32, U1>| if i[0].is_normal() { i[0] } else { 0. },
                    ))))
                }
                "wrap" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| wrap(i[0]))))),
                "mirror" => Some(Net::wrap(Box::new(map(|i: &Frame<f32, U1>| mirror(i[0]))))),
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
            let speed = eval_float(expr.args.first()?, lapis)?;
            let seed = eval_u64(expr.args.get(1)?, lapis)?;
            Some(Net::wrap(Box::new(pebbles(speed, seed))))
        }
        "phaser" => None, //TODO
        "pink" => Some(Net::wrap(Box::new(pink()))),
        "pinkpass" => Some(Net::wrap(Box::new(pinkpass()))),
        "pipe" => {
            let arg0 = expr.args.first()?;
            let x = eval_net(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let y = eval_net(arg1, lapis)?;
            if x.outputs() == y.inputs() {
                Some(x >> y)
            } else {
                None
            }
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
            if x.outputs() == y.outputs() {
                Some(x * y)
            } else {
                None
            }
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
            let delays = eval_vec(arg, lapis)?;
            let time = args.first()?;
            if delays.len() != 32 {
                return None;
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
            let sharpness = eval_float(expr.args.get(1)?, lapis)?;
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
            if x.outputs() == y.outputs() {
                Some(x + y)
            } else {
                None
            }
        }
        "sumf" | "sumi" => None, //TODO
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
        _ => None,
    }
}
