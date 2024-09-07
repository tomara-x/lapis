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
