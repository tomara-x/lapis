use lapis::*;

fn main() {
    // Create a parser instance
    let mut parser = LapisParser::new();

    // Parse a sine wave at 440 Hz
    let mut net = parser.parse_net("sine_hz(440.0)").unwrap();

    // Set sample rate
    let sample_rate = 44100.0;
    let duration = 1.0; // 1 second
    let samples = (sample_rate * duration) as usize;

    // Create a buffer to hold the audio samples
    let mut buffer = vec![0.0f32; samples];

    // Set sample rate for the network
    net.set_sample_rate(sample_rate as f64);
    net.reset();

    // Generate samples
    // The sine generator has 0 inputs and 1 output
    let input = vec![];
    let mut output = vec![0.0f32];

    for sample in buffer.iter_mut() {
        net.tick(&input, &mut output);
        *sample = output[0];
    }

    // Create WAV file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("sine_440hz.wav", spec).unwrap();

    // Write samples to WAV file
    for sample in buffer {
        let amplitude = i16::MAX as f32;
        writer.write_sample((sample.clamp(-1.0, 1.0) * amplitude) as i16).unwrap();
    }

    writer.finalize().unwrap();

    println!("Created sine_440hz.wav - a 1 second 440 Hz sine wave");
}