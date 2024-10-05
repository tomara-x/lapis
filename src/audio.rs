use crate::components::*;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample, Stream,
};
use crossbeam_channel::{bounded, Sender};
use fundsp::hacker32::*;

pub fn default_out_device(slot: SlotBackend) -> Option<Stream> {
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

pub fn set_out_device(lapis: &mut Lapis) {
    if let Some(host_id) = cpal::ALL_HOSTS.get(lapis.out_host.1) {
        if let Ok(host) = cpal::host_from_id(*host_id) {
            if let Ok(mut devices) = host.output_devices() {
                if let Some(device) = devices.nth(lapis.out_device) {
                    if let Ok(default_config) = device.default_output_config() {
                        let mut config = default_config.config();
                        config.channels = 2;
                        let (slot, slot_back) = Slot::new(Box::new(dc(0.) | dc(0.)));
                        lapis.slot = slot;
                        lapis.out_stream = match default_config.sample_format() {
                            cpal::SampleFormat::F32 => run::<f32>(&device, &config, slot_back),
                            cpal::SampleFormat::I16 => run::<i16>(&device, &config, slot_back),
                            cpal::SampleFormat::U16 => run::<u16>(&device, &config, slot_back),
                            format => {
                                eprintln!("unsupported sample format: {}", format);
                                None
                            }
                        };
                    }
                }
            }
        }
    }
}

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

pub fn default_in_device(ls: Sender<f32>, rs: Sender<f32>) -> Option<Stream> {
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

pub fn set_in_device(lapis: &mut Lapis) {
    if let Some(host_id) = cpal::ALL_HOSTS.get(lapis.in_host.1) {
        if let Ok(host) = cpal::host_from_id(*host_id) {
            if let Ok(mut devices) = host.input_devices() {
                if let Some(device) = devices.nth(lapis.in_device) {
                    if let Ok(config) = device.default_input_config() {
                        let (ls, lr) = bounded(4096);
                        let (rs, rr) = bounded(4096);
                        lapis.receivers = (lr, rr);
                        lapis.in_stream = match config.sample_format() {
                            cpal::SampleFormat::F32 => {
                                run_in::<f32>(&device, &config.into(), ls, rs)
                            }
                            cpal::SampleFormat::I16 => {
                                run_in::<i16>(&device, &config.into(), ls, rs)
                            }
                            cpal::SampleFormat::U16 => {
                                run_in::<u16>(&device, &config.into(), ls, rs)
                            }
                            format => {
                                eprintln!("unsupported sample format: {}", format);
                                None
                            }
                        };
                    }
                }
            }
        }
    }
}

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
