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
                        Expr::Binary(expr) => bin_expr_float(&expr),
                        Expr::Paren(expr) => paren_expr_float(&expr.expr),
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
                let segments = &expr.path.segments;
                if let Some(s) = segments.first() {
                    let k = s.ident.to_string();
                    lapis.buffer.push_str(&format!("\n>{:?}", lapis.fmap.get(&k)));
                }
            }
            Expr::Binary(expr) => {
                println!("{:?}", bin_expr_float(&expr));
            }
            _ => {}
        },
        _ => {}
    }
}

fn bin_expr_float(expr: &ExprBinary) -> Option<f32> {
    let left = match *expr.left.clone() {
        Expr::Lit(expr) => lit_float(&expr.lit)?,
        Expr::Binary(expr) => bin_expr_float(&expr)?,
        Expr::Paren(expr) => paren_expr_float(&expr.expr)?,
        _ => return None,
    };
    let right = match *expr.right.clone() {
        Expr::Lit(expr) => lit_float(&expr.lit)?,
        Expr::Binary(expr) => bin_expr_float(&expr)?,
        Expr::Paren(expr) => paren_expr_float(&expr.expr)?,
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

fn paren_expr_float(expr: &Expr) -> Option<f32> {
    match expr {
        Expr::Lit(expr) => lit_float(&expr.lit),
        Expr::Binary(expr) => bin_expr_float(expr),
        _ => None,
    }
}
