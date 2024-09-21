use crate::{components::*, eval::helpers::*, eval::ints::*};
use fundsp::hacker32::*;
use syn::*;

pub fn eval_float(expr: &Expr, lapis: &Lapis) -> Option<f32> {
    match expr {
        Expr::Call(expr) => call_float(expr, lapis),
        Expr::Lit(expr) => lit_float(&expr.lit),
        Expr::Binary(expr) => bin_expr_float(expr, lapis),
        Expr::Paren(expr) => eval_float(&expr.expr, lapis),
        Expr::Path(expr) => path_float(&expr.path, lapis),
        Expr::Unary(expr) => unary_float(expr, lapis),
        Expr::MethodCall(expr) => method_call_float(expr, lapis),
        Expr::Index(expr) => index_float(expr, lapis),
        _ => None,
    }
}

fn index_float(expr: &ExprIndex, lapis: &Lapis) -> Option<f32> {
    let k = nth_path_ident(&expr.expr, 0)?;
    let index = eval_usize(&expr.index, lapis)?;
    lapis.vmap.get(&k)?.get(index).copied()
}

pub fn method_call_float(expr: &ExprMethodCall, lapis: &Lapis) -> Option<f32> {
    match expr.method.to_string().as_str() {
        "value" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let shared = &mut lapis.smap.get(&k)?;
            Some(shared.value())
        }
        "floor" => Some(eval_float(&expr.receiver, lapis)?.floor()),
        "ceil" => Some(eval_float(&expr.receiver, lapis)?.ceil()),
        "round" => Some(eval_float(&expr.receiver, lapis)?.round()),
        "trunc" => Some(eval_float(&expr.receiver, lapis)?.trunc()),
        "fract" => Some(eval_float(&expr.receiver, lapis)?.fract()),
        "abs" => Some(eval_float(&expr.receiver, lapis)?.abs()),
        "signum" => Some(eval_float(&expr.receiver, lapis)?.signum()),
        "copysign" => {
            let sign = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.copysign(sign))
        }
        "div_euclid" => {
            let rhs = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.div_euclid(rhs))
        }
        "rem_euclid" => {
            let rhs = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.rem_euclid(rhs))
        }
        "powi" => {
            let n = eval_i32(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.powi(n))
        }
        "powf" => {
            let n = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.powf(n))
        }
        "sqrt" => Some(eval_float(&expr.receiver, lapis)?.sqrt()),
        "exp" => Some(eval_float(&expr.receiver, lapis)?.exp()),
        "exp2" => Some(eval_float(&expr.receiver, lapis)?.exp2()),
        "ln" => Some(eval_float(&expr.receiver, lapis)?.ln()),
        "log" => {
            let base = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.log(base))
        }
        "log2" => Some(eval_float(&expr.receiver, lapis)?.log2()),
        "log10" => Some(eval_float(&expr.receiver, lapis)?.log10()),
        "cbrt" => Some(eval_float(&expr.receiver, lapis)?.cbrt()),
        "hypot" => {
            let other = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.hypot(other))
        }
        "sin" => Some(eval_float(&expr.receiver, lapis)?.sin()),
        "cos" => Some(eval_float(&expr.receiver, lapis)?.cos()),
        "tan" => Some(eval_float(&expr.receiver, lapis)?.tan()),
        "asin" => Some(eval_float(&expr.receiver, lapis)?.asin()),
        "acos" => Some(eval_float(&expr.receiver, lapis)?.acos()),
        "atan" => Some(eval_float(&expr.receiver, lapis)?.atan()),
        "sinh" => Some(eval_float(&expr.receiver, lapis)?.sinh()),
        "cosh" => Some(eval_float(&expr.receiver, lapis)?.cosh()),
        "tanh" => Some(eval_float(&expr.receiver, lapis)?.tanh()),
        "asinh" => Some(eval_float(&expr.receiver, lapis)?.asinh()),
        "acosh" => Some(eval_float(&expr.receiver, lapis)?.acosh()),
        "atanh" => Some(eval_float(&expr.receiver, lapis)?.atanh()),
        "atan2" => {
            let other = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.atan2(other))
        }
        "recip" => Some(eval_float(&expr.receiver, lapis)?.recip()),
        "to_degrees" => Some(eval_float(&expr.receiver, lapis)?.to_degrees()),
        "to_radians" => Some(eval_float(&expr.receiver, lapis)?.to_radians()),
        "max" => {
            let other = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.max(other))
        }
        "min" => {
            let other = eval_float(expr.args.first()?, lapis)?;
            Some(eval_float(&expr.receiver, lapis)?.min(other))
        }
        "at" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            let arg0 = expr.args.first()?;
            let arg1 = expr.args.get(1)?;
            let chan = eval_usize(arg0, lapis)?;
            let index = eval_usize(arg1, lapis)?;
            if chan < wave.channels() && index < wave.len() {
                Some(wave.at(chan, index))
            } else {
                None
            }
        }
        "channels" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            Some(wave.channels() as f32)
        }
        "len" | "length" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            if let Some(wave) = lapis.wmap.get(&k) {
                Some(wave.len() as f32)
            } else {
                let vec = lapis.vmap.get(&k)?;
                Some(vec.len() as f32)
            }
        }
        "duration" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            Some(wave.duration() as f32)
        }
        "amplitude" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            Some(wave.amplitude())
        }
        "size" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get(&k)?;
            Some(net.size() as f32)
        }
        "inputs" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get(&k)?;
            Some(net.inputs() as f32)
        }
        "outputs" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = lapis.gmap.get(&k)?;
            Some(net.outputs() as f32)
        }
        _ => None,
    }
}
pub fn lit_float(expr: &Lit) -> Option<f32> {
    match expr {
        Lit::Float(expr) => expr.base10_parse::<f32>().ok(),
        Lit::Int(expr) => expr.base10_parse::<f32>().ok(),
        _ => None,
    }
}
pub fn bin_expr_float(expr: &ExprBinary, lapis: &Lapis) -> Option<f32> {
    let left = eval_float(&expr.left, lapis)?;
    let right = eval_float(&expr.right, lapis)?;
    match expr.op {
        BinOp::Sub(_) => Some(left - right),
        BinOp::Div(_) => Some(left / right),
        BinOp::Mul(_) => Some(left * right),
        BinOp::Add(_) => Some(left + right),
        BinOp::Rem(_) => Some(left % right),
        _ => None,
    }
}
pub fn path_float(expr: &Path, lapis: &Lapis) -> Option<f32> {
    let k = expr.segments.first()?.ident.to_string();
    if let Some(c) = constant_float(&k) {
        return Some(c);
    }
    lapis.fmap.get(&k).copied()
}
pub fn unary_float(expr: &ExprUnary, lapis: &Lapis) -> Option<f32> {
    match expr.op {
        UnOp::Neg(_) => Some(-eval_float(&expr.expr, lapis)?),
        _ => None,
    }
}
pub fn call_float(expr: &ExprCall, lapis: &Lapis) -> Option<f32> {
    let func = nth_path_ident(&expr.func, 0)?;
    let args = accumulate_args(&expr.args, lapis);
    match func.as_str() {
        "a_weight" => Some(a_weight(*args.first()?)),
        "abs" => Some(abs(*args.first()?)),
        "amp_db" => Some(amp_db(*args.first()?)),
        "atan" => Some(atan(*args.first()?)),
        "bpm_hz" => Some(bpm_hz(*args.first()?)),
        "ceil" => Some(ceil(*args.first()?)),
        "clamp" => Some(clamp(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "clamp01" => Some(clamp01(*args.first()?)),
        "clamp11" => Some(clamp11(*args.first()?)),
        "cos" => Some(cos(*args.first()?)),
        "cos_hz" => Some(cos_hz(*args.first()?, *args.get(1)?)),
        "cubed" => Some(cubed(*args.first()?)),
        "db_amp" => Some(db_amp(*args.first()?)),
        "delerp" => Some(delerp(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "delerp11" => Some(delerp11(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "dexerp" => Some(dexerp(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "dexerp11" => Some(dexerp11(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "dissonance" => Some(dissonance(*args.first()?, *args.get(1)?)),
        "dissonance_max" => Some(dissonance_max(*args.first()?)),
        "downarc" => Some(downarc(*args.first()?)),
        "ease_noise" => None, //TODO
        "exp" => Some(exp(*args.first()?)),
        "exp2" => Some(exp2(*args.first()?)),
        "exp10" => Some(exp10(*args.first()?)),
        "floor" => Some(floor(*args.first()?)),
        "fractal_ease_noise" => None, //TODO
        "fractal_noise" => {
            let seed = eval_i64(expr.args.first()?, lapis)?;
            let octaves = eval_i64(expr.args.get(1)?, lapis)?;
            let roughness = eval_float(expr.args.get(2)?, lapis)?;
            let x = eval_float(expr.args.get(3)?, lapis)?;
            Some(fractal_noise(seed, octaves, roughness, x))
        }
        "hash1" | "hash2" => None, //TODO
        "identity" => None,        //TODO not useful here
        "lerp" => Some(lerp(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "lerp11" => Some(lerp11(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "log" => Some(log(*args.first()?)),
        "log2" => Some(log2(*args.first()?)),
        "log10" => Some(log10(*args.first()?)),
        "m_weight" => Some(m_weight(*args.first()?)),
        "max" => Some(max(*args.first()?, *args.get(1)?)),
        "midi_hz" => Some(midi_hz(*args.first()?)),
        "min" => Some(min(*args.first()?, *args.get(1)?)),
        "pow" => Some(pow(*args.first()?, *args.get(1)?)),
        "rnd1" => Some(rnd1(eval_u64(expr.args.first()?, lapis)?) as f32),
        "rnd2" => Some(rnd2(eval_u64(expr.args.first()?, lapis)?) as f32),
        "round" => Some(round(*args.first()?)),
        "semitone_ratio" => Some(semitone_ratio(*args.first()?)),
        "signum" => Some(signum(*args.first()?)),
        "sin" => Some(sin(*args.first()?)),
        "sin_hz" => Some(sin_hz(*args.first()?, *args.get(1)?)),
        "sine_ease" => Some(sine_ease(*args.first()?)),
        "smooth3" => Some(smooth3(*args.first()?)),
        "smooth5" => Some(smooth5(*args.first()?)),
        "smooth7" => Some(smooth7(*args.first()?)),
        "smooth9" => Some(smooth9(*args.first()?)),
        "softexp" => Some(softexp(*args.first()?)),
        "softmix" => Some(softmix(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "softsign" => Some(softsign(*args.first()?)),
        "spline" => {
            Some(spline(*args.first()?, *args.get(1)?, *args.get(2)?, *args.get(3)?, *args.get(4)?))
        }
        "spline_mono" => Some(spline_mono(
            *args.first()?,
            *args.get(1)?,
            *args.get(2)?,
            *args.get(3)?,
            *args.get(4)?,
        )),
        "spline_noise" => {
            let seed = eval_u64(expr.args.first()?, lapis)?;
            let x = eval_float(expr.args.get(1)?, lapis)?;
            Some(spline_noise(seed, x))
        }
        "sqr_hz" => Some(sqr_hz(*args.first()?, *args.get(1)?)),
        "sqrt" => Some(sqrt(*args.first()?)),
        "sqared" => Some(squared(*args.first()?)),
        "tan" => Some(tan(*args.first()?)),
        "tanh" => Some(tanh(*args.first()?)),
        "tri_hz" => Some(tri_hz(*args.first()?, *args.get(1)?)),
        "uparc" => Some(uparc(*args.first()?)),
        "xerp" => Some(xerp(*args.first()?, *args.get(1)?, *args.get(2)?)),
        "xerp11" => Some(xerp11(*args.first()?, *args.get(1)?, *args.get(2)?)),
        _ => None,
    }
}

fn constant_float(s: &str) -> Option<f32> {
    match s {
        "E" => Some(std::f32::consts::E),
        "FRAC_1_PI" => Some(std::f32::consts::FRAC_1_PI),
        "FRAC_1_SQRT_2" => Some(std::f32::consts::FRAC_1_SQRT_2),
        "FRAC_2_PI" => Some(std::f32::consts::FRAC_2_PI),
        "FRAC_2_SQRT_PI" => Some(std::f32::consts::FRAC_2_SQRT_PI),
        "FRAC_PI_2" => Some(std::f32::consts::FRAC_PI_2),
        "FRAC_PI_3" => Some(std::f32::consts::FRAC_PI_3),
        "FRAC_PI_4" => Some(std::f32::consts::FRAC_PI_4),
        "FRAC_PI_6" => Some(std::f32::consts::FRAC_PI_6),
        "FRAC_PI_8" => Some(std::f32::consts::FRAC_PI_8),
        "LN_2" => Some(std::f32::consts::LN_2),
        "LN_10" => Some(std::f32::consts::LN_10),
        "LOG2_10" => Some(std::f32::consts::LOG2_10),
        "LOG2_E" => Some(std::f32::consts::LOG2_E),
        "LOG10_2" => Some(std::f32::consts::LOG10_2),
        "LOG10_E" => Some(std::f32::consts::LOG10_E),
        "PI" => Some(std::f32::consts::PI),
        "SQRT_2" => Some(std::f32::consts::SQRT_2),
        "TAU" => Some(std::f32::consts::TAU),
        "EGAMMA" => Some(0.5772157),
        "FRAC_1_SQRT_3" => Some(0.57735026),
        "FRAC_1_SQRT_PI" => Some(0.5641896),
        "PHI" => Some(1.618034),
        "SQRT_3" => Some(1.7320508),
        "inf" | "Inf" | "INF" => Some(f32::INFINITY),
        "nan" | "Nan" | "NaN" | "NAN" => Some(f32::NAN),
        _ => None,
    }
}
