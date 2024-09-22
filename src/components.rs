use crate::audio::*;
use crossbeam_channel::{bounded, Receiver};
use fundsp::hacker32::*;
use std::collections::HashMap;
use std::sync::Arc;

#[allow(dead_code)]
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
    pub slot: Slot,
    pub out_stream: Option<cpal::Stream>,
    pub in_stream: Option<cpal::Stream>,
    pub receivers: (Receiver<f32>, Receiver<f32>),
    pub in_host: usize,
    pub out_host: usize,
    pub in_device: usize,
    pub out_device: usize,
    pub in_device_names: Vec<String>,
    pub out_device_names: Vec<String>,
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
            slot,
            out_stream,
            in_stream,
            receivers: (lr, rr),
            in_host: usize::MAX,
            out_host: usize::MAX,
            in_device: usize::MAX,
            out_device: usize::MAX,
            in_device_names: Vec::new(),
            out_device_names: Vec::new(),
        }
    }
}
