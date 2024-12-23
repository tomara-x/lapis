use crate::{
    components::*,
    eval::{helpers::*, ints::*, nets::*},
};
use fundsp::hacker32::*;
use syn::*;

pub fn method_source(expr: &ExprMethodCall, lapis: &Lapis) -> Option<Source> {
    match expr.method.to_string().as_str() {
        "source" => {
            if let Some(k) = nth_path_ident(&expr.receiver, 0) {
                if let Some(g) = &mut lapis.gmap.get(&k) {
                    let arg0 = expr.args.first();
                    let arg1 = expr.args.get(1);
                    if let (Some(arg0), Some(arg1)) = (arg0, arg1) {
                        let id = path_nodeid(arg0, lapis);
                        let chan = eval_usize(arg1, lapis);
                        if let (Some(id), Some(chan)) = (id, chan) {
                            if g.contains(id) && chan < g.inputs_in(id) {
                                return Some(g.source(id, chan));
                            }
                        }
                    }
                }
            }
            None
        }
        "output_source" => {
            if let Some(k) = nth_path_ident(&expr.receiver, 0) {
                if let Some(g) = &mut lapis.gmap.get(&k) {
                    let arg0 = expr.args.first();
                    if let Some(arg0) = arg0 {
                        let chan = eval_usize(arg0, lapis);
                        if let Some(chan) = chan {
                            return Some(g.output_source(chan));
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

pub fn eval_source(expr: &Expr, lapis: &Lapis) -> Option<Source> {
    match expr {
        Expr::Call(expr) => {
            let seg0 = nth_path_ident(&expr.func, 0)?;
            let seg1 = nth_path_ident(&expr.func, 1)?;
            if seg0 == "Source" {
                if seg1 == "Local" {
                    let arg0 = expr.args.first()?;
                    let arg1 = expr.args.get(1)?;
                    let id = path_nodeid(arg0, lapis)?;
                    let index = eval_usize(arg1, lapis)?;
                    Some(Source::Local(id, index))
                } else if seg1 == "Global" {
                    let arg0 = expr.args.first()?;
                    let index = eval_usize(arg0, lapis)?;
                    Some(Source::Global(index))
                } else {
                    None
                }
            } else {
                None
            }
        }
        Expr::Path(expr) => {
            let seg0 = &expr.path.segments.first()?.ident;
            let seg1 = &expr.path.segments.get(1)?.ident;
            if seg0 == "Source" && seg1 == "Zero" {
                Some(Source::Zero)
            } else {
                None
            }
        }
        Expr::MethodCall(expr) => method_source(expr, lapis),
        _ => None,
    }
}
