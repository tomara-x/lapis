use crate::{components::*, eval::functions::*};
use fundsp::hacker32::*;
use syn::*;

/// Adaptive and ShapeFn are not supported (yet?)

#[derive(Clone)]
pub enum ShapeEnum {
    Atan(Atan),
    Clip(Clip),
    ClipTo(ClipTo),
    Crush(Crush),
    SoftCrush(SoftCrush),
    Softsign(Softsign),
    Tanh(Tanh),
}

impl Shape for ShapeEnum {
    fn shape(&mut self, input: f32) -> f32 {
        match self {
            ShapeEnum::Atan(i) => i.shape(input),
            ShapeEnum::Clip(i) => i.shape(input),
            ShapeEnum::ClipTo(i) => i.shape(input),
            ShapeEnum::Crush(i) => i.shape(input),
            ShapeEnum::SoftCrush(i) => i.shape(input),
            ShapeEnum::Softsign(i) => i.shape(input),
            ShapeEnum::Tanh(i) => i.shape(input),
        }
    }
}

pub fn call_shape(expr: &Expr, lapis: &Lapis) -> Option<ShapeEnum> {
    match expr {
        Expr::Call(expr) => {
            let ident = nth_path_ident(&expr.func, 0)?;
            let args = accumulate_args(&expr.args, lapis);
            match ident.as_str() {
                "Atan" => Some(ShapeEnum::Atan(Atan(*args.first()?))),
                "Clip" => Some(ShapeEnum::Clip(Clip(*args.first()?))),
                "ClipTo" => Some(ShapeEnum::ClipTo(ClipTo(*args.first()?, *args.get(1)?))),
                "Crush" => Some(ShapeEnum::Crush(Crush(*args.first()?))),
                "SoftCrush" => Some(ShapeEnum::SoftCrush(SoftCrush(*args.first()?))),
                "Softsign" => Some(ShapeEnum::Softsign(Softsign(*args.first()?))),
                "Tanh" => Some(ShapeEnum::Tanh(Tanh(*args.first()?))),
                _ => None,
            }
        }
        _ => None,
    }
}
