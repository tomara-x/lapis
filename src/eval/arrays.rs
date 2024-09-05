use crate::{components::*, eval::floats::*};
use syn::*;

pub fn path_arr<'a>(expr: &'a Expr, lapis: &'a Lapis) -> Option<&'a Vec<f32>> {
    match expr {
        Expr::Path(expr) => {
            let k = expr.path.segments.first()?.ident.to_string();
            lapis.vmap.get(&k)
        }
        _ => None,
    }
}
pub fn array_lit(expr: &Expr, lapis: &Lapis) -> Option<Vec<f32>> {
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
