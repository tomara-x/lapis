use crate::{components::*, eval::functions::*};
use fundsp::math::*;
use syn::*;

pub fn half_binary_float(expr: &Expr, lapis: &Lapis) -> Option<f32> {
    match expr {
        Expr::Call(expr) => call_float(expr, lapis),
        Expr::Lit(expr) => lit_float(&expr.lit),
        Expr::Binary(expr) => bin_expr_float(expr, lapis),
        Expr::Paren(expr) => half_binary_float(&expr.expr, lapis),
        Expr::Path(expr) => path_float(&expr.path, lapis),
        Expr::Unary(expr) => unary_float(expr, lapis),
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
    let left = half_binary_float(&expr.left, lapis)?;
    let right = half_binary_float(&expr.right, lapis)?;
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
        UnOp::Neg(_) => Some(-half_binary_float(&expr.expr, lapis)?),
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
