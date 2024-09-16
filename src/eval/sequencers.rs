use crate::{
    components::*,
    eval::{bools::*, floats::*, functions::*, ints::*, nets::*},
};
use fundsp::hacker32::*;
use syn::*;

pub fn call_seq(expr: &Expr, lapis: &Lapis) -> Option<Sequencer> {
    match expr {
        Expr::Call(expr) => {
            let seg0 = nth_path_ident(&expr.func, 0)?;
            if seg0 == "Sequencer" {
                let seg1 = nth_path_ident(&expr.func, 1)?;
                if seg1 == "new" {
                    let arg0 = expr.args.first()?;
                    let arg1 = expr.args.get(1)?;
                    let replay = eval_bool(arg0, lapis)?;
                    let outputs = eval_usize(arg1, lapis)?;
                    return Some(Sequencer::new(replay, outputs));
                }
            }
            None
        }
        _ => None,
    }
}

pub fn seq_methods(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<()> {
    match expr.method.to_string().as_str() {
        "edit" => {
            let id = path_eventid(expr.args.first()?, lapis)?;
            let end_time = eval_float(expr.args.get(1)?, lapis)? as f64;
            let fade_out = eval_float(expr.args.get(2)?, lapis)? as f64;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = &mut lapis.seqmap.get_mut(&k)?;
            seq.edit(id, end_time, fade_out);
        }
        "edit_relative" => {
            let id = path_eventid(expr.args.first()?, lapis)?;
            let end_time = eval_float(expr.args.get(1)?, lapis)? as f64;
            let fade_out = eval_float(expr.args.get(2)?, lapis)? as f64;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = &mut lapis.seqmap.get_mut(&k)?;
            seq.edit_relative(id, end_time, fade_out);
        }
        "set_sample_rate" => {
            let arg = expr.args.first()?;
            let sr = eval_float(arg, lapis)? as f64;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = &mut lapis.seqmap.get_mut(&k)?;
            seq.set_sample_rate(sr);
        }
        "reset" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = &mut lapis.seqmap.get_mut(&k)?;
            seq.reset();
        }
        _ => {}
    }
    None
}

pub fn path_seq<'a>(expr: &'a Expr, lapis: &'a Lapis) -> Option<&'a Sequencer> {
    match expr {
        Expr::Path(expr) => {
            let k = expr.path.segments.first()?.ident.to_string();
            lapis.seqmap.get(&k)
        }
        _ => None,
    }
}

pub fn method_eventid(expr: &Expr, lapis: &mut Lapis) -> Option<EventId> {
    match expr {
        Expr::MethodCall(expr) => match expr.method.to_string().as_str() {
            "push" => {
                let start_time = eval_float(expr.args.first()?, lapis)? as f64;
                let end_time = eval_float(expr.args.get(1)?, lapis)? as f64;
                let fade = path_fade(expr.args.get(2)?)?;
                let fade_in = eval_float(expr.args.get(3)?, lapis)? as f64;
                let fade_out = eval_float(expr.args.get(4)?, lapis)? as f64;
                let unit = Box::new(eval_net(expr.args.get(5)?, lapis)?);
                let k = nth_path_ident(&expr.receiver, 0)?;
                let seq = &mut lapis.seqmap.get_mut(&k)?;
                let duration = end_time - start_time;
                if unit.inputs() != 0
                    || unit.outputs() != seq.outputs()
                    || fade_in > duration
                    || fade_out > duration
                {
                    return None;
                }
                Some(seq.push(start_time, end_time, fade, fade_in, fade_out, unit))
            }
            "push_relative" => {
                let start_time = eval_float(expr.args.first()?, lapis)? as f64;
                let end_time = eval_float(expr.args.get(1)?, lapis)? as f64;
                let fade = path_fade(expr.args.get(2)?)?;
                let fade_in = eval_float(expr.args.get(3)?, lapis)? as f64;
                let fade_out = eval_float(expr.args.get(4)?, lapis)? as f64;
                let unit = Box::new(eval_net(expr.args.get(5)?, lapis)?);
                let k = nth_path_ident(&expr.receiver, 0)?;
                let seq = &mut lapis.seqmap.get_mut(&k)?;
                let duration = end_time - start_time;
                if unit.inputs() != 0
                    || unit.outputs() != seq.outputs()
                    || fade_in > duration
                    || fade_out > duration
                {
                    return None;
                }
                Some(seq.push_relative(start_time, end_time, fade, fade_in, fade_out, unit))
            }
            "push_duration" => {
                let start_time = eval_float(expr.args.first()?, lapis)? as f64;
                let duration = eval_float(expr.args.get(1)?, lapis)? as f64;
                let fade = path_fade(expr.args.get(2)?)?;
                let fade_in = eval_float(expr.args.get(3)?, lapis)? as f64;
                let fade_out = eval_float(expr.args.get(4)?, lapis)? as f64;
                let unit = Box::new(eval_net(expr.args.get(5)?, lapis)?);
                let k = nth_path_ident(&expr.receiver, 0)?;
                let seq = &mut lapis.seqmap.get_mut(&k)?;
                if unit.inputs() != 0
                    || unit.outputs() != seq.outputs()
                    || fade_in > duration
                    || fade_out > duration
                {
                    return None;
                }
                Some(seq.push_duration(start_time, duration, fade, fade_in, fade_out, unit))
            }
            _ => None,
        },
        _ => None,
    }
}

pub fn path_eventid(expr: &Expr, lapis: &Lapis) -> Option<EventId> {
    let k = nth_path_ident(expr, 0)?;
    lapis.eventmap.get(&k).copied()
}
