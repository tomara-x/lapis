use crate::eval::*;

pub fn eval_string(expr: &Expr, lapis: &Lapis) -> Option<String> {
    match expr {
        Expr::Call(expr) => call_string(expr, lapis),
        Expr::Lit(expr) => lit_string(&expr.lit),
        Expr::Path(expr) => path_string(&expr.path, lapis),
        _ => None,
    }
}

fn lit_string(expr: &Lit) -> Option<String> {
    if let Lit::Str(expr) = expr {
        return Some(expr.value());
    }
    None
}

fn path_string(expr: &Path, lapis: &Lapis) -> Option<String> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.string_map.get(&k).cloned()
}

fn call_string(expr: &ExprCall, lapis: &Lapis) -> Option<String> {
    let func = nth_path_ident(&expr.func, 0)?;
    match func.as_str() {
        "file" => {
            let path = eval_string(expr.args.first()?, lapis)?;
            std::fs::read_to_string(path).ok()
        }
        "replace" => {
            let string = eval_string(expr.args.first()?, lapis)?;
            let from = eval_string(expr.args.get(1)?, lapis)?;
            let to = eval_string(expr.args.get(2)?, lapis)?;
            Some(string.replace(&from, &to))
        }
        "replacen" => {
            let string = eval_string(expr.args.first()?, lapis)?;
            let from = eval_string(expr.args.get(1)?, lapis)?;
            let to = eval_string(expr.args.get(2)?, lapis)?;
            let n = eval_usize(expr.args.get(3)?, lapis)?;
            Some(string.replacen(&from, &to, n))
        }
        "format" => {
            let mut string = eval_string(expr.args.first()?, lapis)?;
            let mut iter = expr.args.iter();
            iter.next();
            for arg in iter {
                if let Some(f) = eval_float(arg, lapis) {
                    string = string.replacen("$", &format!("{f}"), 1);
                } else if let Some(s) = eval_string(arg, lapis) {
                    string = string.replacen("$", &s, 1);
                }
            }
            Some(string)
        }
        _ => None,
    }
}
