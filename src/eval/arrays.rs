use crate::{
    components::*,
    eval::{floats::*, helpers::*, ints::*},
};
use syn::*;

pub fn eval_vec_ref<'a>(expr: &'a Expr, lapis: &'a Lapis) -> Option<&'a Vec<f32>> {
    match expr {
        Expr::Path(_) => {
            let k = nth_path_ident(expr, 0)?;
            lapis.vmap.get(&k)
        }
        Expr::MethodCall(expr) => method_call_vec_ref(expr, lapis),
        _ => None,
    }
}

pub fn method_call_vec_ref<'a>(expr: &'a ExprMethodCall, lapis: &'a Lapis) -> Option<&'a Vec<f32>> {
    match expr.method.to_string().as_str() {
        "channel" => {
            let arg = expr.args.first()?;
            let chan = eval_usize(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            if chan < wave.channels() {
                Some(wave.channel(chan))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn array_lit(expr: &ExprArray, lapis: &Lapis) -> Option<Vec<f32>> {
    let mut arr = Vec::new();
    for elem in &expr.elems {
        if let Some(n) = eval_float(elem, lapis) {
            arr.push(n);
        }
    }
    Some(arr)
}

fn method_call_vec(expr: &ExprMethodCall, lapis: &Lapis) -> Option<Vec<f32>> {
    match expr.method.to_string().as_str() {
        "channel" => {
            let arg = expr.args.first()?;
            let chan = eval_usize(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            if chan < wave.channels() {
                Some(wave.channel(chan).clone())
            } else {
                None
            }
        }
        "clone" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            lapis.vmap.get(&k).cloned()
        }
        _ => None,
    }
}

pub fn eval_vec(expr: &Expr, lapis: &mut Lapis) -> Option<Vec<f32>> {
    match expr {
        Expr::Array(expr) => array_lit(expr, lapis),
        Expr::Path(_) => {
            let k = nth_path_ident(expr, 0)?;
            lapis.vmap.remove(&k)
        }
        Expr::MethodCall(expr) => method_call_vec(expr, lapis),
        _ => None,
    }
}

pub fn eval_vec_cloned(expr: &Expr, lapis: &Lapis) -> Option<Vec<f32>> {
    match expr {
        Expr::Array(expr) => array_lit(expr, lapis),
        Expr::Path(_) => {
            let k = nth_path_ident(expr, 0)?;
            lapis.vmap.get(&k).cloned()
        }
        Expr::MethodCall(expr) => method_call_vec(expr, lapis),
        _ => None,
    }
}

pub fn vec_methods(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<()> {
    match expr.method.to_string().as_str() {
        "push" => {
            let arg = expr.args.first()?;
            let v = eval_float(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let vec = &mut lapis.vmap.get_mut(&k)?;
            vec.push(v);
        }
        "pop" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let vec = &mut lapis.vmap.get_mut(&k)?;
            vec.pop();
        }
        "insert" => {
            let index = eval_usize(expr.args.first()?, lapis)?;
            let val = eval_float(expr.args.get(1)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let vec = &mut lapis.vmap.get_mut(&k)?;
            if index < vec.len() {
                vec.insert(index, val);
            }
        }
        "remove" => {
            let index = eval_usize(expr.args.first()?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let vec = &mut lapis.vmap.get_mut(&k)?;
            if index < vec.len() {
                vec.remove(index);
            }
        }
        "resize" => {
            let new_len = eval_usize(expr.args.first()?, lapis)?;
            let val = eval_float(expr.args.get(1)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let vec = &mut lapis.vmap.get_mut(&k)?;
            vec.resize(new_len, val);
        }
        "clear" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let vec = &mut lapis.vmap.get_mut(&k)?;
            vec.clear();
        }
        _ => {}
    }
    None
}
