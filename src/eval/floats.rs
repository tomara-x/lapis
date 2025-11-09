use crate::eval::*;

pub fn eval_float_f32(expr: &Expr, lapis: &Lapis) -> Option<f32> {
    Some(eval_float(expr, lapis)? as f32)
}

pub fn eval_float(expr: &Expr, lapis: &Lapis) -> Option<f64> {
    match expr {
        Expr::Call(expr) => call_float(expr, lapis),
        Expr::Lit(expr) => lit_float(&expr.lit),
        Expr::Binary(expr) => bin_expr_float(expr, lapis),
        Expr::Paren(expr) => eval_float(&expr.expr, lapis),
        Expr::Path(expr) => path_float(&expr.path, lapis),
        Expr::Unary(expr) => unary_float(expr, lapis),
        Expr::MethodCall(expr) => method_float(expr, lapis),
        Expr::Index(expr) => index_float(expr, lapis),
        Expr::Field(expr) => field_float(expr, lapis),
        _ => None,
    }
}

fn field_float(expr: &ExprField, lapis: &Lapis) -> Option<f64> {
    let base = nth_path_ident(&expr.base, 0)?;
    if let Member::Named(ident) = &expr.member {
        let config = if base == "out_stream" {
            &lapis.out_stream.as_ref()?.0
        } else if base == "in_stream" {
            &lapis.in_stream.as_ref()?.0
        } else {
            return None;
        };
        return match ident.to_string().as_str() {
            "sr" => Some(config.sample_rate.0 as f64),
            "chan" => Some(config.channels as f64),
            "buffer" => {
                if let cpal::BufferSize::Fixed(size) = config.buffer_size {
                    return Some(size as f64);
                }
                None
            }
            _ => None,
        };
    }
    None
}

fn index_float(expr: &ExprIndex, lapis: &Lapis) -> Option<f64> {
    let k = nth_path_ident(&expr.expr, 0)?;
    let index = eval_usize(&expr.index, lapis)?;
    Some(*lapis.vmap.get(&k)?.get(index)? as f64)
}

fn method_float(expr: &ExprMethodCall, lapis: &Lapis) -> Option<f64> {
    if let Some(f) = eval_float(&expr.receiver, lapis) {
        match expr.method.to_string().as_str() {
            "floor" => Some(f.floor()),
            "ceil" => Some(f.ceil()),
            "round" => Some(f.round()),
            "trunc" => Some(f.trunc()),
            "fract" => Some(f.fract()),
            "abs" => Some(f.abs()),
            "signum" => Some(f.signum()),
            "copysign" => {
                let sign = eval_float(expr.args.first()?, lapis)?;
                Some(f.copysign(sign))
            }
            "div_euclid" => {
                let rhs = eval_float(expr.args.first()?, lapis)?;
                Some(f.div_euclid(rhs))
            }
            "rem_euclid" => {
                let rhs = eval_float(expr.args.first()?, lapis)?;
                Some(f.rem_euclid(rhs))
            }
            "powi" => {
                let n = eval_i32(expr.args.first()?, lapis)?;
                Some(f.powi(n))
            }
            "powf" => {
                let n = eval_float(expr.args.first()?, lapis)?;
                Some(f.powf(n))
            }
            "sqrt" => Some(f.sqrt()),
            "exp" => Some(f.exp()),
            "exp2" => Some(f.exp2()),
            "ln" => Some(f.ln()),
            "log" => {
                let base = eval_float(expr.args.first()?, lapis)?;
                Some(f.log(base))
            }
            "log2" => Some(f.log2()),
            "log10" => Some(f.log10()),
            "cbrt" => Some(f.cbrt()),
            "hypot" => {
                let other = eval_float(expr.args.first()?, lapis)?;
                Some(f.hypot(other))
            }
            "sin" => Some(f.sin()),
            "cos" => Some(f.cos()),
            "tan" => Some(f.tan()),
            "asin" => Some(f.asin()),
            "acos" => Some(f.acos()),
            "atan" => Some(f.atan()),
            "sinh" => Some(f.sinh()),
            "cosh" => Some(f.cosh()),
            "tanh" => Some(f.tanh()),
            "asinh" => Some(f.asinh()),
            "acosh" => Some(f.acosh()),
            "atanh" => Some(f.atanh()),
            "atan2" => {
                let other = eval_float(expr.args.first()?, lapis)?;
                Some(f.atan2(other))
            }
            "recip" => Some(f.recip()),
            "to_degrees" => Some(f.to_degrees()),
            "to_radians" => Some(f.to_radians()),
            "max" => {
                let other = eval_float(expr.args.first()?, lapis)?;
                Some(f.max(other))
            }
            "min" => {
                let other = eval_float(expr.args.first()?, lapis)?;
                Some(f.min(other))
            }
            _ => None,
        }
    } else if let Some(k) = nth_path_ident(&expr.receiver, 0) {
        match expr.method.to_string().as_str() {
            "value" => {
                let shared = &mut lapis.smap.get(&k)?;
                Some(shared.value() as f64)
            }
            "at" => {
                if let Some(wave) = lapis.wmap.get(&k) {
                    let arg0 = expr.args.first()?;
                    let arg1 = expr.args.get(1)?;
                    let chan = eval_usize(arg0, lapis)?;
                    let index = eval_usize(arg1, lapis)?;
                    if chan < wave.channels() && index < wave.len() {
                        return Some(wave.at(chan, index) as f64);
                    }
                } else if let Some(table) = lapis.atomic_table_map.get(&k) {
                    let i = eval_usize(expr.args.first()?, lapis)?;
                    if i < table.len() {
                        return Some(table.at(i) as f64);
                    }
                }
                None
            }
            "sample_rate" => {
                let wave = lapis.wmap.get(&k)?;
                Some(wave.sample_rate())
            }
            "channels" => {
                let wave = lapis.wmap.get(&k)?;
                Some(wave.channels() as f64)
            }
            "len" | "length" => {
                if let Some(wave) = lapis.wmap.get(&k) {
                    Some(wave.len() as f64)
                } else if let Some(table) = lapis.atomic_table_map.get(&k) {
                    Some(table.len() as f64)
                } else {
                    let vec = lapis.vmap.get(&k)?;
                    Some(vec.len() as f64)
                }
            }
            "duration" => {
                let wave = lapis.wmap.get(&k)?;
                Some(wave.duration())
            }
            "amplitude" => {
                let wave = lapis.wmap.get(&k)?;
                Some(wave.amplitude() as f64)
            }
            "size" => {
                let net = lapis.gmap.get(&k)?;
                Some(net.size() as f64)
            }
            "inputs" => {
                let net = lapis.gmap.get(&k)?;
                Some(net.inputs() as f64)
            }
            "outputs" => {
                let net = lapis.gmap.get(&k)?;
                Some(net.outputs() as f64)
            }
            "inputs_in" => {
                let net = lapis.gmap.get(&k)?;
                let id = eval_path_nodeid(expr.args.first()?, lapis)?;
                if net.contains(id) { Some(net.inputs_in(id) as f64) } else { None }
            }
            "outputs_in" => {
                let net = lapis.gmap.get(&k)?;
                let id = eval_path_nodeid(expr.args.first()?, lapis)?;
                if net.contains(id) { Some(net.outputs_in(id) as f64) } else { None }
            }
            "first" => {
                let vec = &mut lapis.vmap.get(&k)?;
                Some(*vec.first()? as f64)
            }
            "last" => {
                let vec = &mut lapis.vmap.get(&k)?;
                Some(*vec.last()? as f64)
            }
            "get" => {
                let index = eval_usize(expr.args.first()?, lapis)?;
                let vec = &mut lapis.vmap.get(&k)?;
                Some(*vec.get(index)? as f64)
            }
            _ => None,
        }
    } else {
        None
    }
}

fn lit_float(expr: &Lit) -> Option<f64> {
    match expr {
        Lit::Float(expr) => expr.base10_parse::<f64>().ok(),
        Lit::Int(expr) => expr.base10_parse::<f64>().ok(),
        _ => None,
    }
}

fn bin_expr_float(expr: &ExprBinary, lapis: &Lapis) -> Option<f64> {
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

fn path_float(expr: &Path, lapis: &Lapis) -> Option<f64> {
    let k = expr.segments.first()?.ident.to_string();
    if let Some(c) = constant_float(&k) { Some(c) } else { lapis.fmap.get(&k).copied() }
}

fn unary_float(expr: &ExprUnary, lapis: &Lapis) -> Option<f64> {
    match expr.op {
        UnOp::Neg(_) => Some(-eval_float(&expr.expr, lapis)?),
        _ => None,
    }
}

fn call_float(expr: &ExprCall, lapis: &Lapis) -> Option<f64> {
    let func = nth_path_ident(&expr.func, 0)?;
    if func == "time" {
        let epoch = std::time::UNIX_EPOCH;
        let now = std::time::SystemTime::now();
        return Some(now.duration_since(epoch).ok()?.as_millis() as f64);
    }
    let args = accumulate_args_f64(&expr.args, lapis);
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
            let seed = eval_u64(expr.args.first()?, lapis)?;
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
        "rnd1" => Some(rnd1(eval_u64(expr.args.first()?, lapis)?)),
        "rnd2" => Some(rnd2(eval_u64(expr.args.first()?, lapis)?)),
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
        "wrap" => Some(wrap(*args.first()?)),
        "mirror" => Some(mirror(*args.first()?)),
        _ => None,
    }
}

fn constant_float(s: &str) -> Option<f64> {
    match s {
        "E" => Some(std::f64::consts::E),
        "FRAC_1_PI" => Some(std::f64::consts::FRAC_1_PI),
        "FRAC_1_SQRT_2" => Some(std::f64::consts::FRAC_1_SQRT_2),
        "FRAC_2_PI" => Some(std::f64::consts::FRAC_2_PI),
        "FRAC_2_SQRT_PI" => Some(std::f64::consts::FRAC_2_SQRT_PI),
        "FRAC_PI_2" => Some(std::f64::consts::FRAC_PI_2),
        "FRAC_PI_3" => Some(std::f64::consts::FRAC_PI_3),
        "FRAC_PI_4" => Some(std::f64::consts::FRAC_PI_4),
        "FRAC_PI_6" => Some(std::f64::consts::FRAC_PI_6),
        "FRAC_PI_8" => Some(std::f64::consts::FRAC_PI_8),
        "LN_2" => Some(std::f64::consts::LN_2),
        "LN_10" => Some(std::f64::consts::LN_10),
        "LOG2_10" => Some(std::f64::consts::LOG2_10),
        "LOG2_E" => Some(std::f64::consts::LOG2_E),
        "LOG10_2" => Some(std::f64::consts::LOG10_2),
        "LOG10_E" => Some(std::f64::consts::LOG10_E),
        "PI" => Some(std::f64::consts::PI),
        "SQRT_2" => Some(std::f64::consts::SQRT_2),
        "TAU" => Some(std::f64::consts::TAU),
        "EGAMMA" => Some(0.577_215_664_901_532_9),
        "FRAC_1_SQRT_3" => Some(0.577_350_269_189_625_7),
        "FRAC_1_SQRT_PI" => Some(0.564_189_583_547_756_3),
        "PHI" => Some(1.618_033_988_749_895),
        "SQRT_3" => Some(1.732_050_807_568_877_2),
        "inf" | "Inf" | "INF" => Some(f64::INFINITY),
        "nan" | "Nan" | "NaN" | "NAN" => Some(f64::NAN),
        _ => None,
    }
}

pub fn float_bin_assign(expr: &ExprBinary, lapis: &mut Lapis) -> Option<()> {
    let right = eval_float(&expr.right, lapis)?;
    let k = nth_path_ident(&expr.left, 0)?;
    match expr.op {
        BinOp::AddAssign(_) => *lapis.fmap.get_mut(&k)? += right,
        BinOp::SubAssign(_) => *lapis.fmap.get_mut(&k)? -= right,
        BinOp::MulAssign(_) => *lapis.fmap.get_mut(&k)? *= right,
        BinOp::DivAssign(_) => *lapis.fmap.get_mut(&k)? /= right,
        BinOp::RemAssign(_) => *lapis.fmap.get_mut(&k)? %= right,
        _ => {}
    }
    None
}
