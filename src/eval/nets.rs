use crate::{
    components::*,
    eval::{
        arrays::*, atomics::*, floats::*, functions::*, ints::*, meters::*, shapes::*, units::*,
    },
};
use fundsp::hacker32::*;
use syn::*;

pub fn eval_net(expr: &Expr, lapis: &Lapis) -> Option<Net> {
    match expr {
        Expr::Call(expr) => call_net(expr, lapis),
        Expr::Binary(expr) => bin_expr_net(expr, lapis),
        Expr::Paren(expr) => eval_net(&expr.expr, lapis),
        Expr::Path(expr) => path_net(&expr.path, lapis),
        Expr::Unary(expr) => unary_net(expr, lapis),
        _ => None,
    }
}
pub fn bin_expr_net(expr: &ExprBinary, lapis: &Lapis) -> Option<Net> {
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
pub fn unary_net(expr: &ExprUnary, lapis: &Lapis) -> Option<Net> {
    match expr.op {
        UnOp::Neg(_) => Some(-eval_net(&expr.expr, lapis)?),
        UnOp::Not(_) => Some(!eval_net(&expr.expr, lapis)?),
        _ => None,
    }
}
pub fn path_net(expr: &Path, lapis: &Lapis) -> Option<Net> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.gmap.get(&k).cloned()
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
pub fn call_net(expr: &ExprCall, lapis: &Lapis) -> Option<Net> {
    let func = nth_path_ident(&expr.func, 0)?;
    let args = accumulate_args(&expr.args, lapis);
    match func.as_str() {
        "Net" => {
            let f = nth_path_ident(&expr.func, 1)?;
            if f == "new" {
                let ins = args.first()?;
                let outs = args.get(1)?;
                Some(Net::new(*ins as usize, *outs as usize))
            } else {
                None
            }
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
        "ramp_hz_phase" => {
            let f = args.first()?;
            let p = args.get(1)?;
            Some(Net::wrap(Box::new(ramp_hz_phase(*f, *p))))
        }
        "ramp_phase" => {
            let p = args.first()?;
            Some(Net::wrap(Box::new(ramp_phase(*p))))
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
            let delays = array_cloned(arg, lapis)?;
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
        "sine_phase" => {
            let p = args.first()?;
            Some(Net::wrap(Box::new(sine_phase(*p))))
        }
        "sink" => Some(Net::wrap(Box::new(sink()))),
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
            let loop_point = if let Some(arg) = arg2 { eval_usize(arg, lapis) } else { None };
            Some(Net::wrap(Box::new(wavech(&std::sync::Arc::new(wave), chan, loop_point))))
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
            let loop_point = if let Some(arg) = arg4 { eval_usize(arg, lapis) } else { None };
            Some(Net::wrap(Box::new(wavech_at(
                &std::sync::Arc::new(wave),
                chan,
                start,
                end,
                loop_point,
            ))))
        }
        "white" => Some(Net::wrap(Box::new(white()))),
        "zero" => Some(Net::wrap(Box::new(zero()))),
        _ => None,
    }
}
