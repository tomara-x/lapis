use cpal::{
    FromSample, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam_channel::{Receiver, Sender, bounded};
use eframe::egui::{Key, Modifiers};
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
mod waves;
use {
    arrays::*, atomics::*, bools::*, floats::*, helpers::*, ints::*, nets::*, sequencers::*,
    sources::*, statements::*, waves::*,
};

pub struct SliderSettings {
    pub min: f32,
    pub max: f32,
    pub step_by: f64,
    pub speed: f64,
    pub var: String,
}

pub struct Lapis {
    pub buffer: String,
    pub input: String,
    pub settings: bool,
    pub sliders_window: bool,
    pub sliders: Vec<SliderSettings>,
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
    pub atomic_table_map: HashMap<String, Arc<AtomicTable>>,
    pub slot: Slot,
    pub out_stream: Option<(StreamConfig, Stream)>,
    pub in_stream: Option<(StreamConfig, Stream)>,
    pub receiver: Receiver<(usize, f32)>,
    // (modifiers, key, pressed)
    pub keys: HashMap<(Modifiers, Key, bool), String>,
    pub keys_active: bool,
    pub keys_repeat: bool,
    pub zoom_factor: f32,
    pub quiet: bool,
}

impl Lapis {
    pub fn new() -> Self {
        // dummy things
        let (slot, _) = Slot::new(Box::new(dc(0.)));
        let (_, receiver) = bounded(1);
        let mut lapis = Lapis {
            buffer: String::new(),
            input: String::new(),
            settings: false,
            sliders_window: false,
            sliders: Vec::new(),
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
            atomic_table_map: HashMap::new(),
            slot,
            out_stream: None,
            in_stream: None,
            receiver,
            keys: HashMap::new(),
            keys_active: false,
            keys_repeat: false,
            zoom_factor: 1.,
            quiet: false,
        };
        lapis.set_out_device(None, None, None, None, None);
        lapis.set_in_device(None, None, None, None, None);
        lapis
    }
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
    pub fn drop(&mut self, k: &str) {
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
        self.atomic_table_map.remove(k);
    }
    pub fn set_out_device(
        &mut self,
        host: Option<usize>,
        device: Option<usize>,
        channels: Option<u16>,
        sr: Option<u32>,
        buffer: Option<u32>,
    ) -> Option<()> {
        let host = if let Some(h) = host {
            let host_id = cpal::ALL_HOSTS.get(h)?;
            cpal::host_from_id(*host_id).ok()?
        } else {
            cpal::default_host()
        };
        let device = if let Some(d) = device {
            let mut devices = host.output_devices().ok()?;
            devices.nth(d)?
        } else {
            host.default_output_device()?
        };
        let default_config = device.default_output_config().ok()?;
        let sample_format = default_config.sample_format();
        let mut config = default_config.config();

        if let Some(sr) = sr {
            config.sample_rate = cpal::SampleRate(sr);
        }
        if let Some(size) = buffer {
            config.buffer_size = cpal::BufferSize::Fixed(size);
        }
        if let Some(channels) = channels {
            config.channels = channels;
        }
        let mut net = Net::scalar(config.channels as usize, 0.);
        net.allocate();
        let (slot, slot_back) = Slot::new(Box::new(net));

        let stream = match sample_format {
            cpal::SampleFormat::F32 => run_out::<f32>(&device, &config, slot_back),
            cpal::SampleFormat::I16 => run_out::<i16>(&device, &config, slot_back),
            cpal::SampleFormat::U16 => run_out::<u16>(&device, &config, slot_back),
            format => {
                println!("unsupported sample format: {format}");
                None
            }
        };
        if let Some(stream) = stream {
            self.slot = slot;
            self.out_stream = Some((config, stream));
        }
        None
    }
    pub fn set_in_device(
        &mut self,
        host: Option<usize>,
        device: Option<usize>,
        channels: Option<u16>,
        sr: Option<u32>,
        buffer: Option<u32>,
    ) -> Option<()> {
        let host = if let Some(h) = host {
            let host_id = cpal::ALL_HOSTS.get(h)?;
            cpal::host_from_id(*host_id).ok()?
        } else {
            cpal::default_host()
        };
        let device = if let Some(d) = device {
            let mut devices = host.input_devices().ok()?;
            devices.nth(d)?
        } else {
            host.default_input_device()?
        };
        let default_config = device.default_input_config().ok()?;
        let sample_format = default_config.sample_format();
        let mut config = default_config.config();

        if let Some(sr) = sr {
            config.sample_rate = cpal::SampleRate(sr);
        }
        if let Some(size) = buffer {
            config.buffer_size = cpal::BufferSize::Fixed(size);
        }
        if let Some(channels) = channels {
            config.channels = channels;
        }

        let c = config.channels as usize;
        let (s1, r1) = bounded(4096 * c);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => run_in::<f32>(&device, &config, s1),
            cpal::SampleFormat::I16 => run_in::<i16>(&device, &config, s1),
            cpal::SampleFormat::U16 => run_in::<u16>(&device, &config, s1),
            format => {
                println!("unsupported sample format: {format}");
                None
            }
        };
        if let Some(stream) = stream {
            self.in_stream = Some((config, stream));
            self.receiver = r1;
        }
        None
    }
}

fn run_out<T>(device: &cpal::Device, config: &StreamConfig, slot: SlotBackend) -> Option<Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let mut slot = BlockRateAdapter::new(Box::new(slot));
    let channels = config.channels as usize;
    let mut out = vec![0.; channels];

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            for frame in data.chunks_mut(channels) {
                slot.tick(&[], &mut out);
                for i in 0..channels {
                    let tmp = if out[i].is_normal() { out[i].clamp(-1., 1.) } else { 0. };
                    frame[i] = T::from_sample(tmp);
                }
            }
        },
        err_fn,
        None,
    );
    if let Ok(stream) = stream
        && let Ok(()) = stream.play()
    {
        return Some(stream);
    }
    None
}

fn run_in<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    s: Sender<(usize, f32)>,
) -> Option<Stream>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {err}");
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _| {
            for frame in data.chunks(channels) {
                for (channel, sample) in frame.iter().enumerate() {
                    let _ = s.try_send((channel, sample.to_sample::<f32>()));
                }
            }
        },
        err_fn,
        None,
    );
    if let Ok(stream) = stream
        && let Ok(()) = stream.play()
    {
        return Some(stream);
    }
    None
}
