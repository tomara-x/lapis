use crate::eval::*;

pub fn eval_bool(expr: &Expr, lapis: &Lapis) -> Option<bool> {
    match expr {
        Expr::Lit(expr) => lit_bool(&expr.lit),
        Expr::Binary(expr) => bin_expr_bool(expr, lapis),
        Expr::Paren(expr) => eval_bool(&expr.expr, lapis),
        Expr::Path(expr) => path_bool(&expr.path, lapis),
        Expr::Unary(expr) => unary_bool(expr, lapis),
        _ => None,
    }
}

fn lit_bool(expr: &Lit) -> Option<bool> {
    match expr {
        Lit::Bool(expr) => Some(expr.value),
        _ => None,
    }
}

fn bin_expr_bool(expr: &ExprBinary, lapis: &Lapis) -> Option<bool> {
    let left_bool = eval_bool(&expr.left, lapis);
    let right_bool = eval_bool(&expr.right, lapis);
    let left_float = eval_float(&expr.left, lapis);
    let right_float = eval_float(&expr.right, lapis);
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

fn path_bool(expr: &Path, lapis: &Lapis) -> Option<bool> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.bmap.get(&k).copied()
}

fn unary_bool(expr: &ExprUnary, lapis: &Lapis) -> Option<bool> {
    match expr.op {
        UnOp::Not(_) => Some(!eval_bool(&expr.expr, lapis)?),
        _ => None,
    }
}
