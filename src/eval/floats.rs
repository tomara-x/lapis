use crate::{components::*, eval::functions::*, eval::ints::*};
use fundsp::math::*;
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
        _ => None,
    }
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
        "abs" => {
            let n = args.first()?;
            Some(abs(*n))
        }
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
