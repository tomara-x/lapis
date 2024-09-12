use crate::{
    components::*,
    eval::{arrays::*, floats::*, functions::*, ints::*, nets::*},
};
use fundsp::hacker32::*;
use syn::*;

pub fn eval_wave(expr: &Expr, lapis: &mut Lapis) -> Option<Wave> {
    match expr {
        Expr::Call(expr) => call_wave(expr, lapis),
        Expr::MethodCall(expr) => method_wave(expr, lapis),
        _ => None,
    }
}

fn call_wave(expr: &ExprCall, lapis: &Lapis) -> Option<Wave> {
    let seg0 = nth_path_ident(&expr.func, 0)?;
    if seg0 != "Wave" {
        return None;
    }
    let seg1 = nth_path_ident(&expr.func, 1)?;
    match seg1.as_str() {
        "new" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let chans = eval_usize(arg0, lapis)?;
            let sr = eval_float(arg1, lapis)? as f64;
            Some(Wave::new(chans, sr))
        }
        "with_capacity" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let chans = eval_usize(arg0, lapis)?;
            let sr = eval_float(arg1, lapis)? as f64;
            let cap = eval_usize(arg2, lapis)?;
            Some(Wave::with_capacity(chans, sr, cap))
        }
        "zero" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let chans = eval_usize(arg0, lapis)?;
            let sr = eval_float(arg1, lapis)? as f64;
            let dur = eval_float(arg2, lapis)? as f64;
            Some(Wave::zero(chans, sr, dur))
        }
        "from_samples" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let sr = eval_float(arg0, lapis)? as f64;
            if let Some(samps) = eval_arr_ref(arg1, lapis) {
                Some(Wave::from_samples(sr, samps))
            } else {
                array_lit(arg1, lapis).map(|samps| Wave::from_samples(sr, &samps))
            }
        }
        "render" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let sr = eval_float(arg0, lapis)? as f64;
            let dur = eval_float(arg1, lapis)? as f64;
            let mut net = eval_net(arg2, lapis)?;
            if net.inputs() == 0 && net.outputs() > 0 && dur >= 0.0 {
                Some(Wave::render(sr, dur, &mut net))
            } else {
                None
            }
        }
        "render_latency" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let sr = eval_float(arg0, lapis)? as f64;
            let dur = eval_float(arg1, lapis)? as f64;
            let mut net = eval_net(arg2, lapis)?;
            if net.inputs() == 0 && net.outputs() > 0 && dur >= 0.0 {
                Some(Wave::render_latency(sr, dur, &mut net))
            } else {
                None
            }
        }
        "load" => {
            let arg0 = expr.args.first()?;
            if let Expr::Lit(expr) = arg0 {
                if let Lit::Str(expr) = &expr.lit {
                    return Wave::load(expr.value()).ok();
                }
            }
            None
        }
        _ => None,
    }
}

fn method_wave(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<Wave> {
    match expr.method.to_string().as_str() {
        "filter" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let dur = eval_float(arg0, lapis)? as f64;
            let mut node = eval_net(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if node.inputs() == wave.channels() && node.outputs() > 0 && dur >= 0.0 {
                Some(wave.filter(dur, &mut node))
            } else {
                None
            }
        }
        "filter_latency" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let dur = eval_float(arg0, lapis)? as f64;
            let mut node = eval_net(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if node.inputs() == wave.channels() && node.outputs() > 0 && dur >= 0.0 {
                Some(wave.filter_latency(dur, &mut node))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn wave_methods(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<()> {
    match expr.method.to_string().as_str() {
        "set_sample_rate" => {
            let arg0 = expr.args.first()?;
            let sr = eval_float(arg0, lapis)? as f64;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            wave.set_sample_rate(sr)
        }
        "push_channel" => {
            let arg = expr.args.first()?;
            let samps = array_cloned(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if wave.channels() == 0 || wave.len() == samps.len() {
                wave.push_channel(&samps);
            }
        }
        "insert_channel" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let chan = eval_usize(arg0, lapis)?;
            let samps = array_cloned(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if chan <= wave.channels() && (wave.channels() == 0 || wave.len() == samps.len()) {
                wave.insert_channel(chan, &samps);
            }
        }
        "set" => {
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let arg2 = expr.args.get(2)?;
            let chan = eval_usize(arg0, lapis)?;
            let index = eval_usize(arg1, lapis)?;
            let val = eval_float(arg2, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if chan < wave.channels() && index < wave.len() {
                wave.set(chan, index, val);
            }
        }
        "push" => {
            let arg = expr.args.first()?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            if let Expr::Tuple(expr) = arg {
                let p = accumulate_args(&expr.elems, lapis);
                let wave = &mut lapis.wmap.get_mut(&k)?;
                if p.len() == 1 || p.len() == wave.channels() {
                    match p.len() {
                        1 => wave.push(p[0]),
                        2 => wave.push((p[0], p[1])),
                        3 => wave.push((p[0], p[1], p[2])),
                        4 => wave.push((p[0], p[1], p[2], p[3])),
                        5 => wave.push((p[0], p[1], p[2], p[3], p[4])),
                        6 => wave.push((p[0], p[1], p[2], p[3], p[4], p[5])),
                        7 => wave.push((p[0], p[1], p[2], p[3], p[4], p[5], p[6])),
                        8 => wave.push((p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7])),
                        9 => wave.push((p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8])),
                        10 => {
                            wave.push((p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8], p[9]))
                        }
                        _ => {}
                    }
                }
            } else if let Some(val) = eval_float(arg, lapis) {
                let wave = &mut lapis.wmap.get_mut(&k)?;
                wave.push(val);
            }
        }
        "resize" => {
            let arg0 = expr.args.first()?;
            let len = eval_usize(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if wave.channels() > 0 {
                wave.resize(len);
            }
        }
        "normalize" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            wave.normalize();
        }
        "fade_in" => {
            let arg = expr.args.first()?;
            let time = eval_float(arg, lapis)? as f64;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if time <= wave.duration() {
                wave.fade_in(time);
            }
        }
        "fade_out" => {
            let arg = expr.args.first()?;
            let time = eval_float(arg, lapis)? as f64;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if time <= wave.duration() {
                wave.fade_out(time);
            }
        }
        "fade" => {
            let arg = expr.args.first()?;
            let time = eval_float(arg, lapis)? as f64;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = &mut lapis.wmap.get_mut(&k)?;
            if time <= wave.duration() {
                wave.fade(time);
            }
        }
        "save_wav16" => {
            let arg = expr.args.first()?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            if let Expr::Lit(expr) = arg {
                if let Lit::Str(expr) = &expr.lit {
                    let _ = wave.save_wav16(expr.value());
                }
            }
        }
        "save_wav32" => {
            let arg = expr.args.first()?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            if let Expr::Lit(expr) = arg {
                if let Lit::Str(expr) = &expr.lit {
                    let _ = wave.save_wav32(expr.value());
                }
            }
        }
        "remove_channel" => {
            let arg = expr.args.first()?;
            let chan = eval_usize(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get_mut(&k)?;
            if chan < wave.channels() {
                wave.remove_channel(chan);
            }
        }
        _ => {}
    }
    None
}

pub fn path_wave<'a>(expr: &'a Expr, lapis: &'a Lapis) -> Option<&'a Wave> {
    match expr {
        Expr::Path(expr) => {
            let k = expr.path.segments.first()?.ident.to_string();
            lapis.wmap.get(&k)
        }
        _ => None,
    }
}