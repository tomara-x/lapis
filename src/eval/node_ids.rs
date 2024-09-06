use crate::{
    components::*,
    eval::{functions::*, nets::*},
};
use fundsp::hacker32::*;
use syn::*;

pub fn method_nodeid(expr: &Expr, lapis: &mut Lapis) -> Option<NodeId> {
    match expr {
        Expr::MethodCall(expr) => match expr.method.to_string().as_str() {
            "push" => {
                let arg = expr.args.first()?;
                let node = half_binary_net(arg, lapis)?;
                let k = nth_path_ident(&expr.receiver, 0)?;
                let g = &mut lapis.gmap.get_mut(&k)?;
                Some(g.push(Box::new(node)))
            }
            "chain" => {
                let arg = expr.args.first()?;
                let node = half_binary_net(arg, lapis)?;
                let k = nth_path_ident(&expr.receiver, 0)?;
                let g = &mut lapis.gmap.get_mut(&k)?;
                Some(g.chain(Box::new(node)))
            }
            _ => None,
        },
        _ => None,
    }
}
pub fn path_nodeid(expr: &Expr, lapis: &Lapis) -> Option<NodeId> {
    let k = nth_path_ident(expr, 0)?;
    lapis.idmap.get(&k).cloned()
}
