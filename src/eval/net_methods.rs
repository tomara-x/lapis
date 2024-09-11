use crate::{
    components::*,
    eval::{floats::*, functions::*, ints::*, nets::*, node_ids::*},
};
use syn::*;

pub fn net_methods(expr: &ExprMethodCall, lapis: &mut Lapis) -> Option<()> {
    match expr.method.to_string().as_str() {
        "remove" => {
            let arg = expr.args.first()?;
            let id = path_nodeid(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.remove(id);
        }
        "remove_link" => {
            let arg = expr.args.first()?;
            let id = path_nodeid(arg, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.remove_link(id);
        }
        "replace" => {
            let arg0 = expr.args.first()?;
            let id = path_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let unit = eval_net(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.replace(id, Box::new(unit));
        }
        "crossfade" => {
            let arg0 = expr.args.first()?;
            let id = path_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let fade = path_fade(arg1)?;
            let arg2 = expr.args.get(2)?;
            let time = eval_float(arg2, lapis)?;
            let arg3 = expr.args.get(3)?;
            let unit = eval_net(arg3, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.crossfade(id, fade, time, Box::new(unit));
        }
        "connect" => {
            let arg0 = expr.args.first()?;
            let src = path_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let src_port = eval_usize(arg1, lapis)?;
            let arg2 = expr.args.get(2)?;
            let snk = path_nodeid(arg2, lapis)?;
            let arg3 = expr.args.get(3)?;
            let snk_port = eval_usize(arg3, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.connect(src, src_port, snk, snk_port);
        }
        "disconnect" => {
            let arg0 = expr.args.first()?;
            let id = path_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let port = eval_usize(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.disconnect(id, port);
        }
        "connect_input" => {
            let arg0 = expr.args.first()?;
            let global_in = eval_usize(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let snk = path_nodeid(arg1, lapis)?;
            let arg2 = expr.args.get(2)?;
            let snk_port = eval_usize(arg2, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.connect_input(global_in, snk, snk_port);
        }
        "pipe_input" => {
            let arg0 = expr.args.first()?;
            let snk = path_nodeid(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.pipe_input(snk);
        }
        "connect_output" => {
            let arg0 = expr.args.first()?;
            let src = path_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let src_port = eval_usize(arg1, lapis)?;
            let arg2 = expr.args.get(2)?;
            let global_out = eval_usize(arg2, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.connect_output(src, src_port, global_out);
        }
        "disconnect_output" => {
            let arg0 = expr.args.first()?;
            let out = eval_usize(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.disconnect_output(out);
        }
        "pipe_output" => {
            let arg0 = expr.args.first()?;
            let src = path_nodeid(arg0, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.pipe_output(src);
        }
        "pass_through" => {
            let arg0 = expr.args.first()?;
            let input = eval_usize(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let output = eval_usize(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.pass_through(input, output);
        }
        "pipe" => {
            let arg0 = expr.args.first()?;
            let src = path_nodeid(arg0, lapis)?;
            let arg1 = expr.args.get(1)?;
            let snk = path_nodeid(arg1, lapis)?;
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            net.pipe(src, snk);
        }
        "commit" => {
            let k = nth_path_ident(&expr.receiver, 0)?;
            let net = &mut lapis.gmap.get_mut(&k)?;
            if net.has_backend() {
                net.commit();
            }
        }
        _ => {}
    }
    None
}
