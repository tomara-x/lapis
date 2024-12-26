use crate::{
    components::*,
    eval::{floats::*, helpers::*},
};
use fundsp::hacker32::*;
use syn::*;

pub fn eval_shared(expr: &Expr, lapis: &Lapis) -> Option<Shared> {
    match expr {
        Expr::Call(expr) => call_shared(expr, lapis),
        Expr::Path(expr) => path_shared(&expr.path, lapis),
        Expr::Reference(expr) => eval_shared(&expr.expr, lapis),
        _ => None,
    }
}

fn path_shared(expr: &Path, lapis: &Lapis) -> Option<Shared> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.smap.get(&k).cloned()
}

fn call_shared(expr: &ExprCall, lapis: &Lapis) -> Option<Shared> {
    let func = nth_path_ident(&expr.func, 0)?;
    if func == "shared" {
        let arg = expr.args.first()?;
        let val = eval_float(arg, lapis)?;
        Some(shared(val))
    } else {
        None
    }
}

pub fn shared_methods(expr: &ExprMethodCall, lapis: &mut Lapis) {
    if expr.method == "set" || expr.method == "set_value" {
        if let Some(arg) = expr.args.first() {
            if let Some(value) = eval_float(arg, lapis) {
                if let Some(k) = nth_path_ident(&expr.receiver, 0) {
                    if let Some(shared) = &mut lapis.smap.get_mut(&k) {
                        shared.set(value);
                    }
                }
            }
        }
    }
}
