use crate::eval::*;

pub fn call_seq(expr: &Expr, lapis: &Lapis) -> Option<Sequencer> {
    match expr {
        Expr::Call(expr) => {
            let seg0 = nth_path_ident(&expr.func, 0)?;
            if seg0 == "Sequencer" {
                let seg1 = nth_path_ident(&expr.func, 1)?;
                if seg1 == "new" {
                    let replay = eval_bool(expr.args.first()?, lapis)?;
                    let outputs = eval_usize(expr.args.get(1)?, lapis)?;
                    return Some(Sequencer::new(replay, outputs));
                } else if seg1 == "io" {
                    let inputs = eval_usize(expr.args.get(0)?, lapis)?;
                    let outputs = eval_usize(expr.args.get(1)?, lapis)?;
                    return Some(Sequencer::new(false, outputs).with_inputs(inputs));
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
            let id = eval_eventid(expr.args.first()?, lapis)?;
            let end_time = eval_float(expr.args.get(1)?, lapis)?;
            let fade_out = eval_float(expr.args.get(2)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
            seq.edit(id, end_time, fade_out);
        }
        "edit_relative" => {
            let id = eval_eventid(expr.args.first()?, lapis)?;
            let end_time = eval_float(expr.args.get(1)?, lapis)?;
            let fade_out = eval_float(expr.args.get(2)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
            seq.edit_relative(id, end_time, fade_out);
        }
        "set_sample_rate" => {
            let arg = expr.args.first()?;
            let sr = eval_float(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
            seq.set_sample_rate(sr);
        }
        "reset" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
            seq.reset();
        }
        "set_loop" => {
            let start = eval_float(expr.args.get(0)?, lapis)?;
            let end = eval_float(expr.args.get(1)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
            seq.set_loop(start, end);
        }
        "set_time" => {
            let t = eval_float(expr.args.get(0)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
            seq.set_time(t);
        }
        "set_replay_events" => {
            let keep = eval_bool(expr.args.get(0)?, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
            seq.set_replay_events(keep);
        }
        "clear" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
            seq.clear();
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

pub fn eval_eventid(expr: &Expr, lapis: &mut Lapis) -> Option<EventId> {
    match expr {
        Expr::MethodCall(expr) => method_eventid(expr, lapis),
        Expr::Path(expr) => path_eventid(&expr.path, lapis),
        _ => None,
    }
}

fn method_eventid(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<EventId> {
    match expr.method.to_string().as_str() {
        "push" => {
            let start_time = eval_float(expr.args.first()?, lapis)?;
            let end_time = eval_float(expr.args.get(1)?, lapis)?;
            let fade = path_fade(expr.args.get(2)?)?;
            let fade_in = eval_float(expr.args.get(3)?, lapis)?;
            let fade_out = eval_float(expr.args.get(4)?, lapis)?;
            let unit = Box::new(eval_net(expr.args.get(5)?, lapis)?);
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
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
            let start_time = eval_float(expr.args.first()?, lapis)?;
            let end_time = eval_float(expr.args.get(1)?, lapis)?;
            let fade = path_fade(expr.args.get(2)?)?;
            let fade_in = eval_float(expr.args.get(3)?, lapis)?;
            let fade_out = eval_float(expr.args.get(4)?, lapis)?;
            let unit = Box::new(eval_net(expr.args.get(5)?, lapis)?);
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
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
            let start_time = eval_float(expr.args.first()?, lapis)?;
            let duration = eval_float(expr.args.get(1)?, lapis)?;
            let fade = path_fade(expr.args.get(2)?)?;
            let fade_in = eval_float(expr.args.get(3)?, lapis)?;
            let fade_out = eval_float(expr.args.get(4)?, lapis)?;
            let unit = Box::new(eval_net(expr.args.get(5)?, lapis)?);
            let k = nth_path_ident(&expr.receiver, 0)?;
            let seq = lapis.seqmap.get_mut(&k)?;
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
    }
}

fn path_eventid(expr: &Path, lapis: &Lapis) -> Option<EventId> {
    let k = expr.segments.first()?.ident.to_string();
    lapis.eventmap.get(&k).copied()
}
