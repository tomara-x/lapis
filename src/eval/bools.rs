use crate::{components::*, eval::floats::*};
use syn::*;

pub fn half_binary_bool(expr: &Expr, lapis: &Lapis) -> Option<bool> {
    match expr {
        Expr::Lit(expr) => lit_bool(&expr.lit),
        Expr::Binary(expr) => bin_expr_bool(expr, lapis),
        Expr::Paren(expr) => half_binary_bool(&expr.expr, lapis),
        Expr::Path(expr) => path_bool(&expr.path, lapis),
        Expr::Unary(expr) => unary_bool(expr, lapis),
        _ => None,
    }
}
pub fn lit_bool(expr: &Lit) -> Option<bool> {
    match expr {
        Lit::Bool(expr) => Some(expr.value),
        _ => None,
    }
}
pub fn bin_expr_bool(expr: &ExprBinary, lapis: &Lapis) -> Option<bool> {
    let left_bool = half_binary_bool(&expr.left, lapis);
    let right_bool = half_binary_bool(&expr.right, lapis);
    let left_float = half_binary_float(&expr.left, lapis);
    let right_float = half_binary_float(&expr.right, lapis);
    if let (Some(left), Some(right)) = (left_bool, right_bool) {
        match expr.op {
            BinOp::And(_) => Some(left && right),
            BinOp::Or(_) => Some(left || right),
            _ => None,
        }
    } else if let (Some(left), Some(right)) = (left_float, right_float) {
        match expr.op {
            BinOp::Eq(_) => Some(left == right),
            BinOp::Ne(_) => Some(left != right),
            BinOp::Lt(_) => Some(left < right),
            BinOp::Gt(_) => Some(left > right),
            BinOp::Le(_) => Some(left <= right),
            BinOp::Ge(_) => Some(left >= right),
            _ => None,
        }
    } else {
        None
    }
}
pub fn path_bool(expr: &Path, lapis: &Lapis) -> Option<bool> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.bmap.get(&k).copied()
}
pub fn unary_bool(expr: &ExprUnary, lapis: &Lapis) -> Option<bool> {
    match expr.op {
        UnOp::Not(_) => Some(!half_binary_bool(&expr.expr, lapis)?),
        _ => None,
    }
}
