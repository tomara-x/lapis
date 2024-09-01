use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample, Stream,
};
use fundsp::hacker32::*;

pub fn default_out_device(slot: SlotBackend) -> Option<Stream> {
    let host = cpal::default_host();
    if let Some(device) = host.default_output_device() {
        let default_config = device.default_output_config().unwrap();
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
    None
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
