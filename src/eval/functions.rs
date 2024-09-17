use crate::{
    components::*,
    eval::{floats::*, ints::*},
};
use fundsp::hacker32::*;
use syn::punctuated::Punctuated;
use syn::*;

pub fn remove_from_all_maps(k: &String, lapis: &mut Lapis) {
    lapis.fmap.remove(k);
    lapis.vmap.remove(k);
    lapis.gmap.remove(k);
    lapis.idmap.remove(k);
    lapis.bmap.remove(k);
    lapis.smap.remove(k);
    lapis.wmap.remove(k);
    lapis.seqmap.remove(k);
    lapis.eventmap.remove(k);
}
pub fn path_fade(expr: &Expr) -> Option<Fade> {
    let f = nth_path_ident(expr, 0)?;
    let c = nth_path_ident(expr, 1)?;
    if f == "Fade" {
        if c == "Smooth" {
            return Some(Fade::Smooth);
        } else if c == "Power" {
            return Some(Fade::Power);
        }
    }
    None
}
pub fn eval_meter(expr: &Expr, lapis: &Lapis) -> Option<Meter> {
    match expr {
        Expr::Call(expr) => {
            let seg0 = nth_path_ident(&expr.func, 0)?;
            let seg1 = nth_path_ident(&expr.func, 1)?;
            let arg = expr.args.first()?;
            let val = eval_float(arg, lapis)?;
            if seg0 == "Meter" {
                match seg1.as_str() {
                    "Peak" => Some(Meter::Peak(val as f64)),
                    "Rms" => Some(Meter::Rms(val as f64)),
                    _ => None,
                }
            } else {
                None
            }
        }
        Expr::Path(expr) => {
            let seg0 = &expr.path.segments.first()?.ident;
            let seg1 = &expr.path.segments.get(1)?.ident;
            if seg0 == "Meter" && seg1 == "Sample" {
                Some(Meter::Sample)
            } else {
                None
            }
        }
        _ => None,
    }
}
pub fn pat_ident(pat: &Pat) -> Option<String> {
    match pat {
        Pat::Ident(expr) => Some(expr.ident.to_string()),
        Pat::Type(expr) => pat_ident(&expr.pat),
        _ => None,
    }
}
pub fn range_bounds(expr: &Expr, lapis: &Lapis) -> Option<(i32, i32)> {
    match expr {
        Expr::Range(expr) => {
            let start = expr.start.clone()?;
            let end = expr.end.clone()?;
            let s = eval_i32(&start, lapis)?;
            let mut e = eval_i32(&end, lapis)?;
            if let RangeLimits::Closed(_) = expr.limits {
                e += 1;
            }
            Some((s, e))
        }
        _ => None,
    }
}
pub fn nth_path_ident(expr: &Expr, n: usize) -> Option<String> {
    if let Expr::Path(expr) = expr {
        if let Some(expr) = expr.path.segments.get(n) {
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
        if let Some(n) = eval_float(arg, lapis) {
            vec.push(n);
        }
    }
    vec
}
