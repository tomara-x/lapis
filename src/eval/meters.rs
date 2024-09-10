use crate::{components::*, eval::floats::*, eval::functions::*};
use fundsp::hacker32::*;
use syn::*;

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
