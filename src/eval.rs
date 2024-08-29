use crate::components::*;
use syn::*;

pub fn eval(lapis: &mut Lapis) {
    if let Ok(stmt) = parse_str::<Stmt>(&lapis.input) {
        lapis.buffer.push('\n');
        lapis.buffer.push_str(&lapis.input);
        lapis.input.clear();
        println!("{:#?}", stmt);
        if let Stmt::Expr(Expr::Block(expr), _) = stmt {
            for stmt in expr.block.stmts {
                eval_stmt(stmt, lapis);
            }
        } else {
            eval_stmt(stmt, lapis);
        }
    }
}

fn eval_stmt(s: Stmt, lapis: &mut Lapis) {
    match s {
        Stmt::Local(expr) => {
            if let Pat::Ident(i) = expr.pat {
                let k = i.ident.to_string();
                if let Some(expr) = expr.init {
                    let expr = *expr.expr;
                    let v = match expr {
                        Expr::Lit(expr) => lit_float(&expr.lit),
                        Expr::Binary(expr) => bin_expr_float(&expr, lapis),
                        Expr::Paren(expr) => paren_expr_float(&expr.expr, lapis),
                        _ => None,
                    };
                    if let Some(v) = v {
                        lapis.fmap.insert(k, v);
                    }
                }
            }
        }
        Stmt::Expr(expr, _) => match expr {
            Expr::Path(expr) => {
                let n = path_float(&expr.path, lapis);
                lapis.buffer.push_str(&format!("\n>{:?}", n));
            }
            Expr::Binary(expr) => {
                let n = bin_expr_float(&expr, lapis);
                lapis.buffer.push_str(&format!("\n>{:?}", n));
            }
            _ => {}
        },
        _ => {}
    }
}

fn path_float(expr: &Path, lapis: &Lapis) -> Option<f32> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.fmap.get(&k).copied()
}

fn bin_expr_float(expr: &ExprBinary, lapis: &Lapis) -> Option<f32> {
    let left = match *expr.left.clone() {
        Expr::Lit(expr) => lit_float(&expr.lit)?,
        Expr::Binary(expr) => bin_expr_float(&expr, lapis)?,
        Expr::Paren(expr) => paren_expr_float(&expr.expr, lapis)?,
        Expr::Path(expr) => path_float(&expr.path, lapis)?,
        _ => return None,
    };
    let right = match *expr.right.clone() {
        Expr::Lit(expr) => lit_float(&expr.lit)?,
        Expr::Binary(expr) => bin_expr_float(&expr, lapis)?,
        Expr::Paren(expr) => paren_expr_float(&expr.expr, lapis)?,
        Expr::Path(expr) => path_float(&expr.path, lapis)?,
        _ => return None,
    };
    match expr.op {
        BinOp::Sub(_) => Some(left - right),
        BinOp::Div(_) => Some(left / right),
        BinOp::Mul(_) => Some(left * right),
        BinOp::Add(_) => Some(left + right),
        _ => None,
    }
}

fn lit_float(expr: &Lit) -> Option<f32> {
    match expr {
        Lit::Float(expr) => expr.base10_parse::<f32>().ok(),
        Lit::Int(expr) => expr.base10_parse::<f32>().ok(),
        _ => None,
    }
}

fn paren_expr_float(expr: &Expr, lapis: &Lapis) -> Option<f32> {
    match expr {
        Expr::Lit(expr) => lit_float(&expr.lit),
        Expr::Binary(expr) => bin_expr_float(expr, lapis),
        Expr::Path(expr) => path_float(&expr.path, lapis),
        _ => None,
    }
}