use syn::*;

pub fn half_binary_int(expr: &Expr) -> Option<i32> {
    match expr {
        Expr::Lit(expr) => lit_int(&expr.lit),
        Expr::Paren(expr) => half_binary_int(&expr.expr),
        Expr::Unary(expr) => unary_int(expr),
        _ => None,
    }
}
pub fn lit_int(expr: &Lit) -> Option<i32> {
    match expr {
        Lit::Int(expr) => expr.base10_parse::<i32>().ok(),
        _ => None,
    }
}
pub fn unary_int(expr: &ExprUnary) -> Option<i32> {
    match expr.op {
        UnOp::Neg(_) => Some(-half_binary_int(&expr.expr)?),
        _ => None,
    }
}
pub fn lit_u64(expr: &Expr) -> Option<u64> {
    match expr {
        Expr::Lit(expr) => match &expr.lit {
            Lit::Int(expr) => expr.base10_parse::<u64>().ok(),
            _ => None,
        },
        _ => None,
    }
}
