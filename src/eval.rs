use crate::components::*;
use fundsp::hacker32::*;
use syn::punctuated::Punctuated;
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
                        lapis.gmap.remove(&k);
                        lapis.vmap.insert(k, arr);
                    } else if let Some(v) = half_binary_float(&expr.expr, lapis) {
                        lapis.vmap.remove(&k);
                        lapis.gmap.remove(&k);
                        lapis.fmap.insert(k, v);
                    } else if let Some(v) = half_binary_net(&expr.expr, lapis) {
                        lapis.vmap.remove(&k);
                        lapis.fmap.remove(&k);
                        lapis.gmap.insert(k, v);
                    }
                }
            }
        }
        Stmt::Expr(expr, _) => {
            if let Some(n) = half_binary_float(&expr, lapis) {
                lapis.buffer.push_str(&format!("\n>{:?}", n));
            } else if let Some(arr) = path_arr(&expr, lapis) {
                lapis.buffer.push_str(&format!("\n>{:?}", arr));
            } else if let Some(mut g) = half_binary_net(&expr, lapis) {
                lapis.buffer.push_str(&format!("\n{}", g.display()));
            }
        }
        _ => {}
    }
}

// -------------------- nodes --------------------
fn half_binary_net(expr: &Expr, lapis: &Lapis) -> Option<Net> {
    match expr {
        Expr::Call(expr) => call_net(expr, lapis),
        Expr::Binary(expr) => bin_expr_net(expr, lapis),
        Expr::Paren(expr) => half_binary_net(&expr.expr, lapis),
        Expr::Path(expr) => path_net(&expr.path, lapis),
        Expr::Unary(expr) => unary_net(expr, lapis),
        _ => None,
    }
}
fn bin_expr_net(expr: &ExprBinary, lapis: &Lapis) -> Option<Net> {
    let left_net = half_binary_net(&expr.left, lapis);
    let right_net = half_binary_net(&expr.right, lapis);
    let left_float = half_binary_float(&expr.left, lapis);
    let right_float = half_binary_float(&expr.right, lapis);
    if left_net.is_some() && right_net.is_some() {
        let (left, right) = (left_net.unwrap(), right_net.unwrap());
        let (li, lo) = (left.inputs(), left.outputs());
        let (ri, ro) = (right.inputs(), right.outputs());
        match expr.op {
            BinOp::BitAnd(_) if li == ri && lo == ro => Some(left & right),
            BinOp::BitOr(_) => Some(left | right),
            BinOp::BitXor(_) if li == ri => Some(left ^ right),
            BinOp::Shr(_) if lo == ri => Some(left >> right),
            BinOp::Sub(_) if lo == ro => Some(left - right),
            BinOp::Mul(_) if lo == ro => Some(left * right),
            BinOp::Add(_) if lo == ro => Some(left + right),
            _ => None,
        }
    } else if let (Some(left), Some(right)) = (left_net, right_float) {
        match expr.op {
            BinOp::Sub(_) => Some(left - right),
            BinOp::Mul(_) => Some(left * right),
            BinOp::Add(_) => Some(left + right),
            _ => None,
        }
    } else if let (Some(left), Some(right)) = (left_float, right_net) {
        match expr.op {
            BinOp::Sub(_) => Some(left - right),
            BinOp::Mul(_) => Some(left * right),
            BinOp::Add(_) => Some(left + right),
            _ => None,
        }
    } else {
        None
    }
}
fn unary_net(expr: &ExprUnary, lapis: &Lapis) -> Option<Net> {
    match expr.op {
        UnOp::Neg(_) => Some(-half_binary_net(&expr.expr, lapis)?),
        UnOp::Not(_) => Some(!half_binary_net(&expr.expr, lapis)?),
        _ => None,
    }
}
fn path_net(expr: &Path, lapis: &Lapis) -> Option<Net> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.gmap.get(&k).cloned()
}
fn call_net(expr: &ExprCall, lapis: &Lapis) -> Option<Net> {
    let func = path_ident(&expr.func)?;
    let generic = path_generic(&expr.func);
    let args = accumulate_args(&expr.args, lapis);
    println!("{:?}", func);
    println!("{:?}", generic);
    println!("{:?}", args);
    match func.as_str() {
        "dc" => {
            let tuple = expr.args.first()?;
            if let Expr::Tuple(expr) = tuple {
                let p = accumulate_args(&expr.elems, lapis);
                match p.len() {
                    1 => Some(Net::wrap(Box::new(dc(p[0])))),
                    2 => Some(Net::wrap(Box::new(dc((p[0], p[1]))))),
                    3 => Some(Net::wrap(Box::new(dc((p[0], p[1], p[2]))))),
                    4 => Some(Net::wrap(Box::new(dc((p[0], p[1], p[2], p[3]))))),
                    5 => Some(Net::wrap(Box::new(dc((p[0], p[1], p[2], p[3], p[4]))))),
                    6 => Some(Net::wrap(Box::new(dc((p[0], p[1], p[2], p[3], p[4], p[5]))))),
                    7 => Some(Net::wrap(Box::new(dc((p[0], p[1], p[2], p[3], p[4], p[5], p[7]))))),
                    8 => Some(Net::wrap(Box::new(dc((
                        p[0], p[1], p[2], p[3], p[4], p[5], p[7], p[8],
                    ))))),
                    _ => None,
                }
            } else {
                match args.len() {
                    1 => Some(Net::wrap(Box::new(dc(args[0])))),
                    _ => None,
                }
            }
        }
        "sine" => Some(Net::wrap(Box::new(sine()))),
        "lowpass" => Some(Net::wrap(Box::new(lowpass()))),
        _ => None,
    }
}
fn path_ident(expr: &Expr) -> Option<String> {
    if let Expr::Path(expr) = expr {
        if let Some(expr) = expr.path.segments.first() {
            return Some(expr.ident.to_string());
        }
    }
    None
}
fn path_generic(expr: &Expr) -> Option<String> {
    if let Expr::Path(expr) = expr {
        if let Some(expr) = expr.path.segments.first() {
            if let PathArguments::AngleBracketed(expr) = &expr.arguments {
                let args = expr.args.first()?;
                if let GenericArgument::Type(Type::Path(expr)) = args {
                    let expr = expr.path.segments.first()?;
                    return Some(expr.ident.to_string());
                }
            }
        }
    }
    None
}
fn accumulate_args(args: &Punctuated<Expr, Token!(,)>, lapis: &Lapis) -> Vec<f32> {
    let mut vec = Vec::new();
    for arg in args {
        if let Some(n) = half_binary_float(arg, lapis) {
            vec.push(n);
        }
    }
    vec
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
