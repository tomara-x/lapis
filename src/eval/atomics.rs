use crate::eval::*;

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
        let val = eval_float_f32(arg, lapis)?;
        Some(shared(val))
    } else {
        None
    }
}

pub fn shared_methods(expr: &ExprMethodCall, lapis: &Lapis) -> Option<()> {
    if expr.method == "set" || expr.method == "set_value" {
        let k = nth_path_ident(&expr.receiver, 0)?;
        if let Some(shared) = lapis.smap.get(&k) {
            let value = eval_float_f32(expr.args.first()?, lapis)?;
            shared.set(value);
        } else if let Some(table) = lapis.atomic_table_map.get(&k) {
            let i = eval_usize(expr.args.first()?, lapis)?;
            let value = eval_float_f32(expr.args.get(1)?, lapis)?;
            table.set(i, value);
        }
    }
    None
}

pub fn eval_atomic_table(expr: &Expr, lapis: &mut Lapis) -> Option<AtomicTable> {
    if let Expr::Call(expr) = expr {
        let func = nth_path_ident(&expr.func, 0)?;
        if func == "atomic_table" || func == "AtomicTable" {
            let wave = eval_vec(expr.args.first()?, lapis)?;
            if wave.len().is_power_of_two() {
                return Some(AtomicTable::new(&wave));
            }
        }
    }
    None
}
