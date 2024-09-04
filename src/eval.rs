use crate::{components::*, units::*};
use fundsp::hacker32::*;
use syn::punctuated::Punctuated;
use syn::*;

pub fn eval(lapis: &mut Lapis) {
    if let Ok(stmt) = parse_str::<Stmt>(&lapis.input) {
        lapis.buffer.push('\n');
        lapis.buffer.push_str(&lapis.input);
        lapis.input.clear();
        println!("{:#?}", stmt);
        eval_stmt(stmt, lapis);
    }
}

fn eval_stmt(s: Stmt, lapis: &mut Lapis) {
    match s {
        Stmt::Local(expr) => {
            if let Pat::Ident(i) = expr.pat {
                let k = i.ident.to_string();
                if let Some(expr) = expr.init {
                    if let Some(v) = half_binary_float(&expr.expr, lapis) {
                        lapis.vmap.remove(&k);
                        lapis.gmap.remove(&k);
                        lapis.fmap.insert(k, v);
                    } else if let Some(v) = half_binary_net(&expr.expr, lapis) {
                        lapis.vmap.remove(&k);
                        lapis.fmap.remove(&k);
                        lapis.gmap.insert(k, v);
                    } else if let Some(arr) = array_lit(&expr.expr, lapis) {
                        lapis.fmap.remove(&k);
                        lapis.gmap.remove(&k);
                        lapis.vmap.insert(k, arr);
                    }
                }
            }
        }
        Stmt::Expr(expr, _) => match expr {
            Expr::MethodCall(expr) => match expr.method.to_string().as_str() {
                "play" => {
                    if let Some(g) = half_binary_net(&expr.receiver, lapis) {
                        if g.inputs() == 0 && g.outputs() == 1 {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(g | dc(0.)));
                        } else if g.inputs() == 0 && g.outputs() == 2 {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(g));
                        } else {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(dc(0.) | dc(0.)));
                        }
                    }
                }
                "tick" => {
                    // temporary testing implementation. will be refactored
                    if let Some(mut g) = half_binary_net(&expr.receiver, lapis) {
                        if let Some(arr) = expr.args.first() {
                            if let Some(input) = array_lit(&arr, lapis) {
                                let mut output = Vec::new();
                                output.resize(g.outputs(), 0.);
                                g.tick(&input, &mut output);
                                println!("{:?}", output);
                            }
                        }
                    }
                }
                _ => {}
            },
            Expr::Assign(expr) => {
                let Some(ident) = path_ident(&expr.left) else { return };
                if let Some(f) = half_binary_float(&expr.right, lapis) {
                    if let Some(var) = lapis.fmap.get_mut(&ident) {
                        *var = f;
                    }
                } else if let Some(g) = half_binary_net(&expr.right, lapis) {
                    if let Some(var) = lapis.gmap.get_mut(&ident) {
                        *var = g;
                    }
                } else if let Some(a) = array_lit(&expr.right, lapis) {
                    if let Some(var) = lapis.vmap.get_mut(&ident) {
                        *var = a;
                    }
                }
            }
            Expr::ForLoop(expr) => {
                let Some(ident) = pat_ident(&expr.pat) else { return };
                let Some((r0, r1)) = range_bounds(&expr.expr) else { return };
                let tmp = lapis.fmap.remove(&ident);
                for i in r0..r1 {
                    lapis.fmap.insert(ident.clone(), i as f32);
                    for stmt in &expr.body.stmts {
                        eval_stmt(stmt.clone(), lapis);
                    }
                }
                if let Some(old) = tmp {
                    lapis.fmap.insert(ident, old);
                } else {
                    lapis.fmap.remove(&ident);
                }
            }
            Expr::Block(expr) => {
                for stmt in expr.block.stmts {
                    eval_stmt(stmt, lapis);
                }
            }
            _ => {
                if let Some(n) = half_binary_float(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n    {:?}", n));
                } else if let Some(arr) = path_arr(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n    {:?}", arr));
                } else if let Some(mut g) = half_binary_net(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n{}", g.display()));
                    lapis.buffer.push_str(&format!("Size           : {}", g.size()));
                }
            }
        },
        _ => {}
    }
}

// -------------------- chaos --------------------
fn pat_ident(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(expr) => Some(expr.ident.to_string()),
        _ => None,
    }
}
fn range_bounds(expr: &Expr) -> Option<(i32, i32)> {
    match expr {
        Expr::Range(expr) => {
            let start = expr.start.clone()?;
            let end = expr.end.clone()?;
            let s = half_binary_int(&start)?;
            let mut e = half_binary_int(&end)?;
            if let RangeLimits::Closed(_) = expr.limits {
                e += 1;
            }
            Some((s, e))
        }
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
fn nth_path_generic(expr: &Expr, n: usize) -> Option<String> {
    if let Expr::Path(expr) = expr {
        if let Some(expr) = expr.path.segments.first() {
            if let PathArguments::AngleBracketed(expr) = &expr.arguments {
                let args = expr.args.get(n)?;
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
    let args = accumulate_args(&expr.args, lapis);
    println!("{:?}", func);
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
        "split" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(MultiSplitUnit::new(1, n))))
        }
        "join" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(MultiJoinUnit::new(1, n))))
        }
        "multisplit" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let m = nth_path_generic(&expr.func, 1)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(MultiSplitUnit::new(n, m))))
        }
        "multijoin" => {
            let n = nth_path_generic(&expr.func, 0)?.get(1..)?.parse::<usize>().ok()?;
            let m = nth_path_generic(&expr.func, 1)?.get(1..)?.parse::<usize>().ok()?;
            Some(Net::wrap(Box::new(MultiJoinUnit::new(n, m))))
        }
        // TODO
        _ => None,
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
fn array_lit(expr: &Expr, lapis: &Lapis) -> Option<Vec<f32>> {
    match expr {
        Expr::Array(expr) => {
            let mut arr = Vec::new();
            for elem in &expr.elems {
                if let Some(n) = half_binary_float(elem, lapis) {
                    arr.push(n);
                }
            }
            Some(arr)
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
// -------------------- integers --------------------
fn half_binary_int(expr: &Expr) -> Option<i32> {
    match expr {
        Expr::Lit(expr) => lit_int(&expr.lit),
        Expr::Paren(expr) => half_binary_int(&expr.expr),
        Expr::Unary(expr) => unary_int(expr),
        _ => None,
    }
}
fn lit_int(expr: &Lit) -> Option<i32> {
    match expr {
        Lit::Int(expr) => expr.base10_parse::<i32>().ok(),
        _ => None,
    }
}
fn unary_int(expr: &ExprUnary) -> Option<i32> {
    match expr.op {
        UnOp::Neg(_) => Some(-half_binary_int(&expr.expr)?),
        _ => None,
    }
}
