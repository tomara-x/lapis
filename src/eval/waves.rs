use crate::eval::*;
use std::sync::Arc;

pub fn eval_wave(expr: &Expr, lapis: &mut Lapis) -> Option<Arc<Wave>> {
    match expr {
        Expr::Call(expr) => call_wave(expr, lapis),
        Expr::MethodCall(expr) => method_wave(expr, lapis),
        Expr::Path(expr) => path_wave(&expr.path, lapis),
        _ => None,
    }
}

fn call_wave(expr: &ExprCall, lapis: &mut Lapis) -> Option<Arc<Wave>> {
    let seg0 = nth_path_ident(&expr.func, 0)?;
    let seg1 = nth_path_ident(&expr.func, 1)?;
    if seg0 == "Arc" && (seg1 == "new" || seg1 == "clone") {
        return eval_wave(expr.args.first()?, lapis);
    }
    if seg0 != "Wave" {
        return None;
    }
    match seg1.as_str() {
        "new" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let chans = eval_usize(arg0, lapis)?;
            let sr = eval_float(arg1, lapis)?;
            Some(Arc::new(Wave::new(chans, sr)))
        }
        "with_capacity" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let chans = eval_usize(arg0, lapis)?;
            let sr = eval_float(arg1, lapis)?;
            let cap = eval_usize(arg2, lapis)?;
            Some(Arc::new(Wave::with_capacity(chans, sr, cap)))
        }
        "zero" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let chans = eval_usize(arg0, lapis)?;
            let sr = eval_float(arg1, lapis)?;
            let dur = eval_float(arg2, lapis)?;
            Some(Arc::new(Wave::zero(chans, sr, dur)))
        }
        "from_samples" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let sr = eval_float(arg0, lapis)?;
            let samps = eval_vec(arg1, lapis)?;
            Some(Arc::new(Wave::from_samples(sr, &samps)))
        }
        "render" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let sr = eval_float(arg0, lapis)?;
            let dur = eval_float(arg1, lapis)?;
            let mut net = eval_net(arg2, lapis)?;
            if net.inputs() == 0 && net.outputs() > 0 && dur >= 0.0 {
                Some(Arc::new(Wave::render(sr, dur, &mut net)))
            } else {
                None
            }
        }
        "render_latency" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let sr = eval_float(arg0, lapis)?;
            let dur = eval_float(arg1, lapis)?;
            let mut net = eval_net(arg2, lapis)?;
            if net.inputs() == 0 && net.outputs() > 0 && dur >= 0.0 {
                Some(Arc::new(Wave::render_latency(sr, dur, &mut net)))
            } else {
                None
            }
        }
        "load" => {
            let arg0 = expr.args.first()?;
            if let Expr::Lit(expr) = arg0
                && let Lit::Str(expr) = &expr.lit
            {
                return Some(Arc::new(Wave::load(expr.value()).ok()?));
            }
            None
        }
        _ => None,
    }
}

fn method_wave(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<Arc<Wave>> {
    match expr.method.to_string().as_str() {
        "filter" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let dur = eval_float(arg0, lapis)?;
            let mut node = eval_net(arg1, lapis)?;
            let wave = eval_wave(&expr.receiver, lapis)?;
            if node.inputs() == wave.channels() && node.outputs() > 0 && dur >= 0.0 {
                Some(Arc::new(wave.filter(dur, &mut node)))
            } else {
                None
            }
        }
        "filter_latency" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let dur = eval_float(arg0, lapis)?;
            let mut node = eval_net(arg1, lapis)?;
            let wave = eval_wave(&expr.receiver, lapis)?;
            if node.inputs() == wave.channels() && node.outputs() > 0 && dur >= 0.0 {
                Some(Arc::new(wave.filter_latency(dur, &mut node)))
            } else {
                None
            }
        }
        "clone" => eval_wave(&expr.receiver, lapis),
        _ => None,
    }
}

fn arc_mut(arc: &mut Arc<Wave>, safe: bool) -> &mut Wave {
    if safe {
        Arc::make_mut(arc)
    } else {
        let ptr = Arc::as_ptr(arc).cast_mut();
        // SAFETY: it's not :3
        unsafe { &mut *ptr }
    }
}

pub fn wave_methods(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<()> {
    let mut s = expr.method.to_string();
    let mut safe = true;
    if let Some(stripped) = s.strip_prefix("unsafe_") {
        s = stripped.to_string();
        safe = false;
    }
    match s.as_str() {
        "set_sample_rate" => {
            let arg0 = expr.args.first()?;
            let sr = eval_float(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            arc_mut(wave, safe).set_sample_rate(sr);
        }
        "push_channel" => {
            let arg = expr.args.first()?;
            let samps = eval_vec(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if wave.channels() == 0 || wave.len() == samps.len() {
                arc_mut(wave, safe).push_channel(&samps);
            }
        }
        "insert_channel" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let chan = eval_usize(arg0, lapis)?;
            let samps = eval_vec(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if chan <= wave.channels() && (wave.channels() == 0 || wave.len() == samps.len()) {
                arc_mut(wave, safe).insert_channel(chan, &samps);
            }
        }
        "mix_channel" => {
            let chan = eval_usize(expr.args.first()?, lapis)?;
            let offset = eval_isize(expr.args.get(1)?, lapis)?;
            let samps = eval_vec(expr.args.get(2)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if chan < wave.channels() {
                arc_mut(wave, safe).mix_channel(chan, offset, &samps);
            }
        }
        "set" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let chan = eval_usize(arg0, lapis)?;
            let index = eval_usize(arg1, lapis)?;
            let val = eval_float_f32(arg2, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if chan < wave.channels() && index < wave.len() {
                arc_mut(wave, safe).set(chan, index, val);
            }
        }
        "mix" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let chan = eval_usize(arg0, lapis)?;
            let index = eval_usize(arg1, lapis)?;
            let val = eval_float_f32(arg2, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if chan < wave.channels() && index < wave.len() {
                arc_mut(wave, safe).mix(chan, index, val);
            }
        }
        "push" => {
            let arg = expr.args.first()?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            if let Expr::Tuple(expr) = arg {
                let p = accumulate_args(&expr.elems, lapis);
                let wave = lapis.wmap.get_mut(&k)?;
                if p.len() == 1 || p.len() == wave.channels() {
                    match p.len() {
                        1 => arc_mut(wave, safe).push(p[0]),
                        2 => arc_mut(wave, safe).push((p[0], p[1])),
                        3 => arc_mut(wave, safe).push((p[0], p[1], p[2])),
                        4 => arc_mut(wave, safe).push((p[0], p[1], p[2], p[3])),
                        5 => arc_mut(wave, safe).push((p[0], p[1], p[2], p[3], p[4])),
                        6 => arc_mut(wave, safe).push((p[0], p[1], p[2], p[3], p[4], p[5])),
                        7 => arc_mut(wave, safe).push((p[0], p[1], p[2], p[3], p[4], p[5], p[6])),
                        8 => arc_mut(wave, safe)
                            .push((p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7])),
                        9 => arc_mut(wave, safe)
                            .push((p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8])),
                        10 => arc_mut(wave, safe)
                            .push((p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8], p[9])),
                        _ => {}
                    }
                }
            } else if let Some(val) = eval_float_f32(arg, lapis) {
                let wave = lapis.wmap.get_mut(&k)?;
                arc_mut(wave, safe).push(val);
            }
        }
        "resize" => {
            let arg0 = expr.args.first()?;
            let len = eval_usize(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if wave.channels() > 0 {
                arc_mut(wave, safe).resize(len);
            }
        }
        "normalize" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            arc_mut(wave, safe).normalize();
        }
        "fade_in" => {
            let arg = expr.args.first()?;
            let time = eval_float(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if time <= wave.duration() {
                arc_mut(wave, safe).fade_in(time);
            }
        }
        "fade_out" => {
            let arg = expr.args.first()?;
            let time = eval_float(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if time <= wave.duration() {
                arc_mut(wave, safe).fade_out(time);
            }
        }
        "fade" => {
            let arg = expr.args.first()?;
            let time = eval_float(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if time <= wave.duration() {
                arc_mut(wave, safe).fade(time);
            }
        }
        "save_wav16" => {
            let name = eval_string(expr.args.first()?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            if wave.channels() > 0 {
                let _ = wave.save_wav16(name);
            }
        }
        "save_wav32" => {
            let name = eval_string(expr.args.first()?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            if wave.channels() > 0 {
                let _ = wave.save_wav32(name);
            }
        }
        "remove_channel" => {
            let arg = expr.args.first()?;
            let chan = eval_usize(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if chan < wave.channels() {
                arc_mut(wave, safe).remove_channel(chan);
            }
        }
        "append" => {
            let arg = expr.args.first()?;
            let src = lapis.wmap.get(&nth_path_ident(arg, 0)?)?.clone();
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if wave.channels() == src.channels() {
                arc_mut(wave, safe).append(&src);
            }
        }
        "retain" => {
            let start = eval_isize(expr.args.first()?, lapis)?;
            let length = eval_usize(expr.args.get(1)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            arc_mut(wave, safe).retain(start, length);
        }
        "amplify" => {
            let amp = eval_float_f32(expr.args.first()?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            arc_mut(wave, safe).amplify(amp);
        }
        _ => {}
    }
    None
}

fn path_wave(expr: &Path, lapis: &Lapis) -> Option<Arc<Wave>> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.wmap.get(&k).cloned()
}
