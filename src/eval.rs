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
                    if let Expr::Array(expr) = *expr.expr {
                        let mut arr = Vec::new();
                        for elem in expr.elems {
                            if let Some(n) = half_binary_float(&elem, lapis) {
                                arr.push(n);
                            }
                        }
                        lapis.fmap.remove(&k);
                        lapis.vmap.insert(k, arr);
                    } else if let Some(v) = half_binary_float(&expr.expr, lapis) {
                        lapis.vmap.remove(&k);
                        lapis.fmap.insert(k, v);
                    }
                }
            }
        }
        Stmt::Expr(expr, _) => {
            if let Some(n) = half_binary_float(&expr, lapis) {
                lapis.buffer.push_str(&format!("\n>{:?}", n));
            } else if let Some(arr) = path_arr(&expr, lapis) {
                lapis.buffer.push_str(&format!("\n>{:?}", arr));
            }
        }
        _ => {}
    }
}

// -------------------- arrays --------------------
fn path_arr<'a>(expr: &'a Expr, lapis: &'a Lapis) -> Option<&'a Vec<f32>> {
    match expr {
        Expr::Path(expr) => {
            let k = expr.path.segments.first()?.ident.to_string();
            lapis.vmap.get(&k)
        }
        _ => None,
    }
}

// -------------------- floats --------------------
fn half_binary_float(expr: &Expr, lapis: &Lapis) -> Option<f32> {
    match expr {
        Expr::Lit(expr) => lit_float(&expr.lit),
        Expr::Binary(expr) => bin_expr_float(expr, lapis),
        Expr::Paren(expr) => half_binary_float(&expr.expr, lapis),
        Expr::Path(expr) => path_float(&expr.path, lapis),
        Expr::Unary(expr) => unary_float(expr, lapis),
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

fn bin_expr_float(expr: &ExprBinary, lapis: &Lapis) -> Option<f32> {
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

fn path_float(expr: &Path, lapis: &Lapis) -> Option<f32> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.fmap.get(&k).copied()
}

fn unary_float(expr: &ExprUnary, lapis: &Lapis) -> Option<f32> {
    match expr.op {
        UnOp::Neg(_) => Some(-half_binary_float(&expr.expr, lapis)?),
        _ => None,
    }
}
