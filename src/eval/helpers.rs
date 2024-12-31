use crate::eval::*;
use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui::{Key, KeyboardShortcut, Modifiers};
use syn::punctuated::Punctuated;

pub fn device_commands(expr: ExprCall, lapis: &mut Lapis) -> Option<()> {
    let func = nth_path_ident(&expr.func, 0)?;
    match func.as_str() {
        "list_in_devices" => {
            let hosts = cpal::platform::ALL_HOSTS;
            lapis.buffer.push_str("\n// input devices:\n");
            for (i, host) in hosts.iter().enumerate() {
                lapis.buffer.push_str(&format!("// {}: {:?}:\n", i, host));
                if let Ok(devices) = cpal::platform::host_from_id(*host).unwrap().input_devices() {
                    for (j, device) in devices.enumerate() {
                        lapis.buffer.push_str(&format!("//     {}: {:?}\n", j, device.name()));
                    }
                }
            }
        }
        "list_out_devices" => {
            let hosts = cpal::platform::ALL_HOSTS;
            lapis.buffer.push_str("\n// output devices:\n");
            for (i, host) in hosts.iter().enumerate() {
                lapis.buffer.push_str(&format!("// {}: {:?}:\n", i, host));
                if let Ok(devices) = cpal::platform::host_from_id(*host).unwrap().output_devices() {
                    for (j, device) in devices.enumerate() {
                        lapis.buffer.push_str(&format!("//     {}: {:?}\n", j, device.name()));
                    }
                }
            }
        }
        "set_in_device" => {
            let h = eval_usize(expr.args.first()?, lapis)?;
            let d = eval_usize(expr.args.get(1)?, lapis)?;
            set_in_device(h, d, lapis);
        }
        "set_out_device" => {
            let h = eval_usize(expr.args.first()?, lapis)?;
            let d = eval_usize(expr.args.get(1)?, lapis)?;
            set_out_device(h, d, lapis);
        }
        _ => {}
    }
    None
}
pub fn parse_shortcut(mut k: String) -> Option<KeyboardShortcut> {
    k = k.replace(char::is_whitespace, "");
    let mut modifiers = Modifiers::NONE;
    if k.contains("Ctrl") || k.contains("ctrl") {
        modifiers = modifiers.plus(Modifiers::CTRL);
    }
    if k.contains("Alt") || k.contains("alt") {
        modifiers = modifiers.plus(Modifiers::ALT);
    }
    if k.contains("Shift") || k.contains("shift") {
        modifiers = modifiers.plus(Modifiers::SHIFT);
    }
    if k.contains("Command") || k.contains("command") {
        modifiers = modifiers.plus(Modifiers::COMMAND);
    }
    k = k
        .replace("Ctrl+", "")
        .replace("ctrl+", "")
        .replace("Alt+", "")
        .replace("alt+", "")
        .replace("Shift+", "")
        .replace("shift+", "")
        .replace("Command+", "")
        .replace("command+", "");
    let key = Key::from_name(&k)?;
    Some(KeyboardShortcut::new(modifiers, key))
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
        Pat::Wild(_) => Some(String::from("_")),
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

// shapes. Adaptive and ShapeFn are not supported (yet?)
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
