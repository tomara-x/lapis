use crate::audio::*;
use crossbeam_channel::{bounded, Receiver};
use eframe::egui::KeyboardShortcut;
use fundsp::hacker32::*;
use std::collections::HashMap;
use std::sync::Arc;
use syn::*;

mod arrays;
mod atomics;
mod bools;
mod floats;
mod helpers;
mod ints;
mod nets;
mod sequencers;
mod sources;
mod units;
mod waves;
use {
    arrays::*, atomics::*, bools::*, floats::*, helpers::*, ints::*, nets::*, sequencers::*,
    sources::*, units::*, waves::*,
};

pub struct Lapis {
    pub buffer: String,
    pub input: String,
    pub settings: bool,
    pub about: bool,
    pub fmap: HashMap<String, f32>,
    pub vmap: HashMap<String, Vec<f32>>,
    pub gmap: HashMap<String, Net>,
    pub idmap: HashMap<String, NodeId>,
    pub bmap: HashMap<String, bool>,
    pub smap: HashMap<String, Shared>,
    pub wmap: HashMap<String, Arc<Wave>>,
    pub seqmap: HashMap<String, Sequencer>,
    pub eventmap: HashMap<String, EventId>,
    pub srcmap: HashMap<String, Source>,
    pub slot: Slot,
    pub out_stream: Option<cpal::Stream>,
    pub in_stream: Option<cpal::Stream>,
    pub receivers: (Receiver<f32>, Receiver<f32>),
    pub keys: Vec<(KeyboardShortcut, String)>,
    pub keys_active: bool,
    pub zoom_factor: f32,
}

impl Lapis {
    pub fn new() -> Self {
        let (slot, slot_back) = Slot::new(Box::new(dc(0.) | dc(0.)));
        let out_stream = default_out_device(slot_back);
        let (ls, lr) = bounded(4096);
        let (rs, rr) = bounded(4096);
        let in_stream = default_in_device(ls, rs);
        Lapis {
            buffer: String::new(),
            input: String::new(),
            settings: false,
            about: false,
            fmap: HashMap::new(),
            vmap: HashMap::new(),
            gmap: HashMap::new(),
            idmap: HashMap::new(),
            bmap: HashMap::new(),
            smap: HashMap::new(),
            wmap: HashMap::new(),
            seqmap: HashMap::new(),
            eventmap: HashMap::new(),
            srcmap: HashMap::new(),
            slot,
            out_stream,
            in_stream,
            receivers: (lr, rr),
            keys: Vec::new(),
            keys_active: false,
            zoom_factor: 1.,
        }
    }
    pub fn eval(&mut self, input: &str) {
        if !input.is_empty() {
            self.buffer.push('\n');
            self.buffer.push_str(input);
            match parse_str::<Stmt>(&format!("{{{}}}", input)) {
                Ok(stmt) => {
                    eval_stmt(stmt, self);
                }
                Err(err) => {
                    self.buffer.push_str(&format!("\n// error: {}", err));
                }
            }
        }
    }
    pub fn eval_input(&mut self) {
        if !self.input.is_empty() {
            match parse_str::<Stmt>(&format!("{{{}}}", self.input)) {
                Ok(stmt) => {
                    self.buffer.push('\n');
                    self.buffer.push_str(&std::mem::take(&mut self.input));
                    eval_stmt(stmt, self);
                }
                Err(err) => {
                    self.buffer.push_str(&format!("\n// error: {}", err));
                }
            }
        }
    }
    pub fn drop(&mut self, k: &String) {
        self.fmap.remove(k);
        self.vmap.remove(k);
        self.gmap.remove(k);
        self.idmap.remove(k);
        self.bmap.remove(k);
        self.smap.remove(k);
        self.wmap.remove(k);
        self.seqmap.remove(k);
        self.eventmap.remove(k);
        self.srcmap.remove(k);
    }
}

#[allow(clippy::map_entry)]
fn eval_stmt(s: Stmt, lapis: &mut Lapis) {
    match s {
        Stmt::Local(expr) => {
            if let Some(k) = pat_ident(&expr.pat) {
                if let Some(expr) = expr.init {
                    if let Some(v) = eval_float(&expr.expr, lapis) {
                        lapis.drop(&k);
                        lapis.fmap.insert(k, v);
                    } else if let Some(v) = eval_net(&expr.expr, lapis) {
                        lapis.drop(&k);
                        lapis.gmap.insert(k, v);
                    } else if let Some(arr) = eval_vec(&expr.expr, lapis) {
                        lapis.drop(&k);
                        lapis.vmap.insert(k, arr);
                    } else if let Some(id) =
                        method_nodeid(&expr.expr, lapis).or(path_nodeid(&expr.expr, lapis))
                    {
                        lapis.drop(&k);
                        lapis.idmap.insert(k, id);
                    } else if let Some(b) = eval_bool(&expr.expr, lapis) {
                        lapis.drop(&k);
                        lapis.bmap.insert(k, b);
                    } else if let Some(s) = eval_shared(&expr.expr, lapis) {
                        lapis.drop(&k);
                        lapis.smap.insert(k, s);
                    } else if let Some(w) = eval_wave(&expr.expr, lapis) {
                        lapis.drop(&k);
                        let wave = Arc::new(w);
                        lapis.wmap.insert(k, wave);
                    } else if let Some(seq) = call_seq(&expr.expr, lapis) {
                        lapis.drop(&k);
                        lapis.seqmap.insert(k, seq);
                    } else if let Some(source) = eval_source(&expr.expr, lapis) {
                        lapis.drop(&k);
                        lapis.srcmap.insert(k, source);
                    } else if let Some(event) =
                        method_eventid(&expr.expr, lapis).or(path_eventid(&expr.expr, lapis))
                    {
                        lapis.drop(&k);
                        lapis.eventmap.insert(k, event);
                    }
                }
            }
        }
        Stmt::Expr(expr, _) => match expr {
            Expr::MethodCall(ref method) => match method.method.to_string().as_str() {
                "play" => {
                    if let Some(g) = eval_net(&method.receiver, lapis) {
                        if g.inputs() == 0 && g.outputs() == 1 {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(g | dc(0.)));
                        } else if g.inputs() == 0 && g.outputs() == 2 {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(g));
                        } else {
                            lapis.slot.set(Fade::Smooth, 0.01, Box::new(dc(0.) | dc(0.)));
                        }
                    }
                }
                "tick" => {
                    let Some(input) = method.args.first() else { return };
                    let Some(in_arr) = eval_vec_cloned(input, lapis) else { return };
                    let mut output = Vec::new();
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                            if g.inputs() != in_arr.len() {
                                return;
                            }
                            output.resize(g.outputs(), 0.);
                            g.tick(&in_arr, &mut output);
                        }
                    } else if let Some(mut g) = eval_net(&method.receiver, lapis) {
                        if g.inputs() != in_arr.len() {
                            return;
                        }
                        output.resize(g.outputs(), 0.);
                        g.tick(&in_arr, &mut output);
                    }
                    if let Some(out) = method.args.get(1) {
                        if let Some(k) = nth_path_ident(out, 0) {
                            if let Some(var) = lapis.vmap.get_mut(&k) {
                                *var = output;
                            }
                        }
                    } else {
                        lapis.buffer.push_str(&format!("\n// {:?}", output));
                    }
                }
                "play_backend" => {
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                            if !g.has_backend() {
                                let g = g.backend();
                                if g.inputs() == 0 && g.outputs() == 2 {
                                    lapis.slot.set(Fade::Smooth, 0.01, Box::new(g));
                                }
                            }
                        } else if let Some(seq) = &mut lapis.seqmap.get_mut(&k) {
                            if !seq.has_backend() {
                                let backend = seq.backend();
                                if backend.outputs() == 2 {
                                    lapis.slot.set(Fade::Smooth, 0.01, Box::new(backend));
                                }
                            }
                        }
                    }
                }
                "drop" => {
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        lapis.drop(&k);
                    }
                }
                "error" => {
                    if let Some(k) = nth_path_ident(&method.receiver, 0) {
                        if let Some(g) = &mut lapis.gmap.get_mut(&k) {
                            lapis.buffer.push_str(&format!("\n// {:?}", g.error()));
                        }
                    }
                }
                _ => {
                    if let Some(n) = method_call_float(method, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", n));
                    } else if let Some(arr) = method_call_vec_ref(method, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", arr));
                    } else if let Some(nodeid) = method_nodeid(&expr, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", nodeid));
                    } else if let Some(event) = method_eventid(&expr, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", event));
                    } else if let Some(mut g) = method_net(method, lapis) {
                        let info = g.display().replace('\n', "\n// ");
                        lapis.buffer.push_str(&format!("\n// {}", info));
                        lapis.buffer.push_str(&format!("Size           : {}", g.size()));
                    } else if let Some(source) = method_source(method, lapis) {
                        lapis.buffer.push_str(&format!("\n// {:?}", source));
                    } else {
                        wave_methods(method, lapis);
                        net_methods(method, lapis);
                        vec_methods(method, lapis);
                        shared_methods(method, lapis);
                        seq_methods(method, lapis);
                    }
                }
            },
            Expr::Assign(expr) => match *expr.left {
                Expr::Path(_) => {
                    let Some(ident) = nth_path_ident(&expr.left, 0) else { return };
                    if let Some(f) = eval_float(&expr.right, lapis) {
                        if let Some(var) = lapis.fmap.get_mut(&ident) {
                            *var = f;
                        }
                    } else if lapis.gmap.contains_key(&ident) {
                        if let Some(g) = eval_net(&expr.right, lapis) {
                            lapis.gmap.insert(ident, g);
                        }
                    } else if lapis.vmap.contains_key(&ident) {
                        if let Some(a) = eval_vec(&expr.right, lapis) {
                            lapis.vmap.insert(ident, a);
                        }
                    } else if let Some(id) =
                        method_nodeid(&expr.right, lapis).or(path_nodeid(&expr.right, lapis))
                    {
                        if let Some(var) = lapis.idmap.get_mut(&ident) {
                            *var = id;
                        }
                    } else if let Some(b) = eval_bool(&expr.right, lapis) {
                        if let Some(var) = lapis.bmap.get_mut(&ident) {
                            *var = b;
                        }
                    } else if let Some(s) = eval_shared(&expr.right, lapis) {
                        if let Some(var) = lapis.smap.get_mut(&ident) {
                            *var = s;
                        }
                    } else if let Some(s) = eval_source(&expr.right, lapis) {
                        if let Some(var) = lapis.srcmap.get_mut(&ident) {
                            *var = s;
                        }
                    } else if let Some(event) =
                        method_eventid(&expr.right, lapis).or(path_eventid(&expr.right, lapis))
                    {
                        if let Some(var) = lapis.eventmap.get_mut(&ident) {
                            *var = event;
                        }
                    }
                }
                Expr::Index(left) => {
                    if let Some(k) = nth_path_ident(&left.expr, 0) {
                        if let Some(index) = eval_usize(&left.index, lapis) {
                            if let Some(right) = eval_float(&expr.right, lapis) {
                                if let Some(vec) = lapis.vmap.get_mut(&k) {
                                    if let Some(v) = vec.get_mut(index) {
                                        *v = right;
                                    }
                                }
                            }
                        }
                    }
                }
                Expr::Lit(left) => {
                    if let Lit::Str(left) = left.lit {
                        if let Expr::Lit(right) = *expr.right {
                            if let Some(shortcut) = parse_shortcut(left.value()) {
                                lapis.keys.retain(|x| x.0 != shortcut);
                                if let Lit::Str(right) = right.lit {
                                    let code = right.value();
                                    if !code.is_empty() {
                                        lapis.keys.push((shortcut, code));
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Expr::ForLoop(expr) => {
                let Some(ident) = pat_ident(&expr.pat) else { return };
                let bounds = range_bounds(&expr.expr, lapis);
                let arr = eval_vec(&expr.expr, lapis);
                let tmp = lapis.fmap.remove(&ident);
                if let Some((r0, r1)) = bounds {
                    for i in r0..r1 {
                        lapis.fmap.insert(ident.clone(), i as f32);
                        for stmt in &expr.body.stmts {
                            eval_stmt(stmt.clone(), lapis);
                        }
                    }
                } else if let Some(arr) = arr {
                    for i in arr {
                        lapis.fmap.insert(ident.clone(), i);
                        for stmt in &expr.body.stmts {
                            eval_stmt(stmt.clone(), lapis);
                        }
                    }
                }
                if let Some(old) = tmp {
                    lapis.fmap.insert(ident, old);
                } else {
                    lapis.fmap.remove(&ident);
                }
            }
            Expr::If(expr) => {
                if let Some(cond) = eval_bool(&expr.cond, lapis) {
                    if cond {
                        let expr = Expr::Block(ExprBlock {
                            attrs: Vec::new(),
                            label: None,
                            block: expr.then_branch,
                        });
                        eval_stmt(Stmt::Expr(expr, None), lapis);
                    } else if let Some((_, else_branch)) = expr.else_branch {
                        eval_stmt(Stmt::Expr(*else_branch, None), lapis);
                    }
                }
            }
            Expr::Block(expr) => {
                for stmt in expr.block.stmts {
                    eval_stmt(stmt, lapis);
                }
            }
            _ => {
                if let Some(n) = eval_float(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", n));
                } else if let Some(arr) = eval_vec_ref(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", arr));
                } else if let Some(arr) = eval_vec_cloned(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", arr));
                } else if let Some(mut g) = eval_net_cloned(&expr, lapis) {
                    let info = g.display().replace('\n', "\n// ");
                    lapis.buffer.push_str(&format!("\n// {}", info));
                    lapis.buffer.push_str(&format!("Size           : {}", g.size()));
                } else if let Some(id) = path_nodeid(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", id));
                } else if let Some(b) = eval_bool(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", b));
                } else if let Some(s) = eval_shared(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// Shared({})", s.value()));
                } else if let Some(w) = path_wave(&expr, lapis) {
                    lapis.buffer.push_str(&format!(
                        "\n// Wave(ch:{}, sr:{}, len:{}, dur:{})",
                        w.channels(),
                        w.sample_rate(),
                        w.len(),
                        w.duration()
                    ));
                } else if let Some(w) = eval_wave(&expr, lapis) {
                    lapis.buffer.push_str(&format!(
                        "\n// Wave(ch:{}, sr:{}, len:{}, dur:{})",
                        w.channels(),
                        w.sample_rate(),
                        w.len(),
                        w.duration()
                    ));
                } else if let Some(seq) = path_seq(&expr, lapis).or(call_seq(&expr, lapis).as_ref())
                {
                    let info = format!(
                        "\n// Sequencer(outs: {}, has_backend: {}, replay: {})",
                        seq.outputs(),
                        seq.has_backend(),
                        seq.replay_events()
                    );
                    lapis.buffer.push_str(&info);
                } else if let Some(source) = eval_source(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", source));
                } else if let Some(event) = path_eventid(&expr, lapis) {
                    lapis.buffer.push_str(&format!("\n// {:?}", event));
                } else if let Expr::Call(expr) = expr {
                    device_commands(expr, lapis);
                } else if let Expr::Binary(expr) = expr {
                    float_bin_assign(&expr, lapis);
                }
            }
        },
        _ => {}
    }
}
