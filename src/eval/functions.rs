use crate::{
    components::*,
    eval::{floats::*, ints::*},
};
use syn::punctuated::Punctuated;
use syn::*;

pub fn pat_ident(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(expr) => Some(expr.ident.to_string()),
        _ => None,
    }
}
pub fn range_bounds(expr: &Expr) -> Option<(i32, i32)> {
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
pub fn path_ident(expr: &Expr) -> Option<String> {
    if let Expr::Path(expr) = expr {
        if let Some(expr) = expr.path.segments.first() {
            return Some(expr.ident.to_string());
        }
    }
    None
}
pub fn nth_path_generic(expr: &Expr, n: usize) -> Option<String> {
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
pub fn accumulate_args(args: &Punctuated<Expr, Token!(,)>, lapis: &Lapis) -> Vec<f32> {
    let mut vec = Vec::new();
    for arg in args {
        if let Some(n) = half_binary_float(arg, lapis) {
            vec.push(n);
        }
    }
    vec
}
