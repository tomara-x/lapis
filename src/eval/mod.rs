#[cfg(feature = "gui")]
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample, Stream,
};
use crossbeam_channel::{bounded, Receiver, Sender};
#[cfg(feature = "gui")]
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
mod statements;
mod units;
mod waves;
use {
    arrays::*, atomics::*, bools::*, floats::*, helpers::*, ints::*, nets::*, sequencers::*,
    sources::*, statements::*, units::*, waves::*,
};

// Re-export key functions for library users
pub use nets::{eval_net, eval_net_cloned};
pub use statements::eval_stmt;

pub struct Lapis {
    #[cfg(feature = "gui")]
    pub buffer: String,
    #[cfg(feature = "gui")]
    pub input: String,
    #[cfg(feature = "gui")]
    pub settings: bool,
    #[cfg(feature = "gui")]
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
    #[cfg(feature = "gui")]
    pub slot: Slot,
    #[cfg(feature = "gui")]
    pub out_stream: Option<cpal::Stream>,
    #[cfg(feature = "gui")]
    pub in_stream: Option<cpal::Stream>,
    pub receivers: (Receiver<f32>, Receiver<f32>),
    #[cfg(feature = "gui")]
    pub keys: Vec<(KeyboardShortcut, String)>,
    #[cfg(feature = "gui")]
    pub keys_active: bool,
    #[cfg(feature = "gui")]
    pub zoom_factor: f32,
    #[cfg(feature = "gui")]
    pub quiet: bool,
}

impl Lapis {
    pub fn new() -> Self {
        #[cfg(feature = "gui")]
        let (slot, slot_back) = Slot::new(Box::new(dc(0.) | dc(0.)));
        #[cfg(feature = "gui")]
        let out_stream = default_out_device(slot_back);
        let (ls, lr) = bounded(4096);
        let (rs, rr) = bounded(4096);
        #[cfg(feature = "gui")]
        let in_stream = default_in_device(ls, rs.clone());
        Lapis {
            #[cfg(feature = "gui")]
            buffer: String::new(),
            #[cfg(feature = "gui")]
            input: String::new(),
            #[cfg(feature = "gui")]
            settings: false,
            #[cfg(feature = "gui")]
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
            #[cfg(feature = "gui")]
            slot,
            #[cfg(feature = "gui")]
            out_stream,
            #[cfg(feature = "gui")]
            in_stream,
            receivers: (lr, rr),
            #[cfg(feature = "gui")]
            keys: Vec::new(),
            #[cfg(feature = "gui")]
            keys_active: false,
            #[cfg(feature = "gui")]
            zoom_factor: 1.,
            #[cfg(feature = "gui")]
            quiet: false,
        }
    }
    #[cfg(feature = "gui")]
    pub fn eval(&mut self, input: &str) {
        if !input.is_empty() {
            self.buffer.push('\n');
            self.buffer.push_str(input);
            match parse_str::<Stmt>(&format!("{{{}}}", input)) {
                Ok(stmt) => {
                    let out = eval_stmt(stmt, self);
                    self.buffer.push_str(&out);
                }
                Err(err) => {
                    self.buffer.push_str(&format!("\n// error: {}", err));
                }
            }
        }
    }
    #[cfg(feature = "gui")]
    pub fn eval_input(&mut self) {
        if !self.input.is_empty() {
            match parse_str::<Stmt>(&format!("{{{}}}", self.input)) {
                Ok(stmt) => {
                    self.buffer.push('\n');
                    self.buffer.push_str(&std::mem::take(&mut self.input));
                    let out = eval_stmt(stmt, self);
                    self.buffer.push_str(&out);
                }
                Err(err) => {
                    self.buffer.push_str(&format!("\n// error: {}", err));
                }
            }
        }
    }
    pub fn quiet_eval(&mut self, input: &str) {
        if let Ok(stmt) = parse_str::<Stmt>(&format!("{{{}}}", input)) {
            eval_stmt(stmt, self);
        }
    }

    /// Library-friendly method to parse and get a Net
    pub fn parse_net(&mut self, input: &str) -> std::result::Result<Net, String> {
        // Try to parse as an expression directly
        match parse_str::<Expr>(input) {
            Ok(expr) => {
                if let Some(net) = eval_net(&expr, self) {
                    Ok(net)
                } else {
                    Err("Expression does not evaluate to a Net".to_string())
                }
            }
            Err(_) => {
                // Fall back to parsing as a statement
                match parse_str::<Stmt>(&format!("{{{}}}", input)) {
                    Ok(stmt) => {
                        match stmt {
                            Stmt::Expr(expr, _) => {
                                if let Some(net) = eval_net(&expr, self) {
                                    Ok(net)
                                } else {
                                    Err("Expression does not evaluate to a Net".to_string())
                                }
                            }
                            _ => Err("Not an expression statement".to_string()),
                        }
                    }
                    Err(err) => Err(format!("Parse error: {}", err)),
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

#[cfg(feature = "gui")]
fn default_out_device(slot: SlotBackend) -> Option<Stream> {
    let host = cpal::default_host();
    if let Some(device) = host.default_output_device() {
        if let Ok(default_config) = device.default_output_config() {
            let mut config = default_config.config();
            config.channels = 2;
            return match default_config.sample_format() {
                cpal::SampleFormat::F32 => run::<f32>(&device, &config, slot),
                cpal::SampleFormat::I16 => run::<i16>(&device, &config, slot),
                cpal::SampleFormat::U16 => run::<u16>(&device, &config, slot),
                format => {
                    eprintln!("unsupported sample format: {}", format);
                    None
                }
            };
        }
    }
    None
}

#[cfg(feature = "gui")]
fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    slot: SlotBackend,
) -> Option<cpal::Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let mut slot = BlockRateAdapter::new(Box::new(slot));

    let mut next_value = move || {
        let (l, r) = slot.get_stereo();
        (
            if l.is_normal() { l.clamp(-1., 1.) } else { 0. },
            if r.is_normal() { r.clamp(-1., 1.) } else { 0. },
        )
    };
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| write_data(data, &mut next_value),
        err_fn,
        None,
    );
    if let Ok(stream) = stream {
        if let Ok(()) = stream.play() {
            return Some(stream);
        }
    }
    None
}

#[cfg(feature = "gui")]
fn write_data<T>(output: &mut [T], next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(2) {
        let sample = next_sample();
        frame[0] = T::from_sample(sample.0);
        frame[1] = T::from_sample(sample.1);
    }
}

#[cfg(feature = "gui")]
fn default_in_device(ls: Sender<f32>, rs: Sender<f32>) -> Option<Stream> {
    let host = cpal::default_host();
    if let Some(device) = host.default_input_device() {
        if let Ok(config) = device.default_input_config() {
            return match config.sample_format() {
                cpal::SampleFormat::F32 => run_in::<f32>(&device, &config.into(), ls, rs),
                cpal::SampleFormat::I16 => run_in::<i16>(&device, &config.into(), ls, rs),
                cpal::SampleFormat::U16 => run_in::<u16>(&device, &config.into(), ls, rs),
                format => {
                    eprintln!("unsupported sample format: {}", format);
                    None
                }
            };
        }
    }
    None
}

#[cfg(feature = "gui")]
fn run_in<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    ls: Sender<f32>,
    rs: Sender<f32>,
) -> Option<cpal::Stream>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            read_data(data, channels, ls.clone(), rs.clone())
        },
        err_fn,
        None,
    );
    if let Ok(stream) = stream {
        if let Ok(()) = stream.play() {
            return Some(stream);
        }
    }
    None
}

#[cfg(feature = "gui")]
fn read_data<T>(input: &[T], channels: usize, ls: Sender<f32>, rs: Sender<f32>)
where
    T: SizedSample,
    f32: FromSample<T>,
{
    for frame in input.chunks(channels) {
        for (channel, sample) in frame.iter().enumerate() {
            if channel & 1 == 0 {
                let _ = ls.try_send(sample.to_sample::<f32>());
            } else {
                let _ = rs.try_send(sample.to_sample::<f32>());
            }
        }
    }
}
