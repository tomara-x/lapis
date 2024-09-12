use crate::{
    components::*,
    eval::{floats::*, functions::*, ints::*},
};
use syn::*;

pub fn eval_arr_ref<'a>(expr: &'a Expr, lapis: &'a Lapis) -> Option<&'a Vec<f32>> {
    match expr {
        Expr::Path(expr) => {
            let k = expr.path.segments.first()?.ident.to_string();
            lapis.vmap.get(&k)
        }
        Expr::MethodCall(expr) => method_call_arr_ref(expr, lapis),
        _ => None,
    }
}

pub fn method_call_arr_ref<'a>(expr: &'a ExprMethodCall, lapis: &'a Lapis) -> Option<&'a Vec<f32>> {
    match expr.method.to_string().as_str() {
        "channel" => {
            let arg = expr.args.first()?;
            let chan = eval_usize(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let wave = lapis.wmap.get(&k)?;
            if chan < wave.channels() {
                return Some(wave.channel(chan));
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn array_lit(expr: &Expr, lapis: &Lapis) -> Option<Vec<f32>> {
    match expr {
        Expr::Array(expr) => {
            let mut arr = Vec::new();
            for elem in &expr.elems {
                if let Some(n) = eval_float(elem, lapis) {
                    arr.push(n);
                }
            }
            Some(arr)
        }
        _ => None,
    }
}
pub fn array_cloned(expr: &Expr, lapis: &Lapis) -> Option<Vec<f32>> {
    match expr {
        Expr::Array(_) => array_lit(expr, lapis),
        Expr::Path(expr) => {
            let k = expr.path.segments.first()?.ident.to_string();
            lapis.vmap.get(&k).cloned()
        }
        Expr::MethodCall(expr) => {
            if expr.method == "channel" {
                let arg = expr.args.first()?;
                let chan = eval_usize(arg, lapis)?;
                let k = nth_path_ident(&expr.receiver, 0)?;
                let wave = lapis.wmap.get(&k)?;
                if chan < wave.channels() {
                    return Some(wave.channel(chan).clone());
                }
            }
            None
        }
        _ => None,
    }
}
