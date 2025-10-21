use crate::eval::*;

pub fn eval_i32(expr: &Expr, lapis: &Lapis) -> Option<i32> {
    let i = match expr {
        Expr::Lit(expr) => lit_i32(&expr.lit),
        Expr::Paren(expr) => eval_i32(&expr.expr, lapis),
        Expr::Unary(expr) => unary_i32(expr, lapis),
        _ => None,
    };
    if i.is_some() { i } else { Some(eval_float(expr, lapis)? as i32) }
}

fn lit_i32(expr: &Lit) -> Option<i32> {
    match expr {
        Lit::Int(expr) => expr.base10_parse::<i32>().ok(),
        _ => None,
    }
}

fn unary_i32(expr: &ExprUnary, lapis: &Lapis) -> Option<i32> {
    match expr.op {
        UnOp::Neg(_) => Some(-eval_i32(&expr.expr, lapis)?),
        _ => None,
    }
}

pub fn eval_i64(expr: &Expr, lapis: &Lapis) -> Option<i64> {
    let i = match expr {
        Expr::Lit(expr) => lit_i64(&expr.lit),
        Expr::Paren(expr) => eval_i64(&expr.expr, lapis),
        Expr::Unary(expr) => unary_i64(expr, lapis),
        _ => None,
    };
    if i.is_some() { i } else { Some(eval_float(expr, lapis)? as i64) }
}

fn lit_i64(expr: &Lit) -> Option<i64> {
    match expr {
        Lit::Int(expr) => expr.base10_parse::<i64>().ok(),
        _ => None,
    }
}

fn unary_i64(expr: &ExprUnary, lapis: &Lapis) -> Option<i64> {
    match expr.op {
        UnOp::Neg(_) => Some(-eval_i64(&expr.expr, lapis)?),
        _ => None,
    }
}

pub fn eval_u64(expr: &Expr, lapis: &Lapis) -> Option<u64> {
    let i = match expr {
        Expr::Lit(expr) => match &expr.lit {
            Lit::Int(expr) => expr.base10_parse::<u64>().ok(),
            _ => None,
        },
        _ => None,
    };
    if i.is_some() { i } else { Some(eval_float(expr, lapis)? as u64) }
}

pub fn eval_usize(expr: &Expr, lapis: &Lapis) -> Option<usize> {
    let i = match expr {
        Expr::Lit(expr) => match &expr.lit {
            Lit::Int(expr) => expr.base10_parse::<usize>().ok(),
            _ => None,
        },
        _ => None,
    };
    if i.is_some() { i } else { Some(eval_float(expr, lapis)? as usize) }
}

pub fn eval_isize(expr: &Expr, lapis: &Lapis) -> Option<isize> {
    let i = match expr {
        Expr::Lit(expr) => match &expr.lit {
            Lit::Int(expr) => expr.base10_parse::<isize>().ok(),
            _ => None,
        },
        _ => None,
    };
    if i.is_some() { i } else { Some(eval_float(expr, lapis)? as isize) }
}
