use crate::audio::*;
use fundsp::hacker32::*;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Lapis {
    pub buffer: String,
    pub input: String,
    pub settings: bool,
    pub about: bool,
    pub maps: bool,
    pub fmap: HashMap<String, f32>,
    pub vmap: HashMap<String, Vec<f32>>,
    pub gmap: HashMap<String, Net>,
    pub idmap: HashMap<String, NodeId>,
    pub bmap: HashMap<String, bool>,
    pub smap: HashMap<String, Shared>,
    pub wmap: HashMap<String, Wave>,
    pub slot: Slot,
    pub stream: Option<cpal::Stream>,
}

impl Lapis {
    pub fn new() -> Self {
        let (slot, slot_back) = Slot::new(Box::new(dc(0.) | dc(0.)));
        let stream = default_out_device(slot_back);
        Lapis {
            buffer: String::new(),
            input: String::new(),
            settings: false,
            about: false,
            maps: false,
            fmap: HashMap::new(),
            vmap: HashMap::new(),
            gmap: HashMap::new(),
            idmap: HashMap::new(),
            bmap: HashMap::new(),
            smap: HashMap::new(),
            wmap: HashMap::new(),
            slot,
            stream,
        }
    }
}
