use cpal::{
    SampleFormat, FromSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

#[derive(Clone)]
pub enum WaveForm {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

pub struct Oscillator {
    frequency: f32,
    pub(crate) sample_rate: u32,
    phase: f32,
    waveform: WaveForm,
    volume: f32,
}

impl Oscillator {
    pub fn new(frequency: f32, waveform: WaveForm, volume: f32) -> Self {
        Self { frequency, sample_rate: 0, phase: 0.0, waveform, volume }
    }

    fn next_sample(&mut self) -> f32 {
        let sample = match self.waveform {
            WaveForm::Sine => (2.0 * std::f32::consts::PI * self.phase).sin(),
            WaveForm::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            WaveForm::Triangle => 4.0 * (self.phase - (self.phase + 0.25).floor()).abs() - 1.0,
            WaveForm::Sawtooth => 2.0 * self.phase - 1.0,
        };
        self.phase = (self.phase + self.frequency / self.sample_rate as f32) % 1.0;
        sample * self.volume
    }
}

fn write_samples<T: cpal::Sample + FromSample<f32>>(osc: &mut Oscillator, output: &mut [T], channels: usize) {
    for frame in output.chunks_mut(channels) {
        let s = T::from_sample(osc.next_sample());
        for sample in frame.iter_mut() {
            *sample = s;
        }
    }
}

fn build_stream(device: &cpal::Device, config: &cpal::SupportedStreamConfig, mut osc: Oscillator) -> cpal::Stream {
    let channels = config.channels() as usize;
    let cfg: cpal::StreamConfig = config.clone().into();
    match config.sample_format() {
        SampleFormat::I8 => device.build_output_stream(&cfg, move |out: &mut [i8], _| write_samples(&mut osc, out, channels), |_| {}, None),
        SampleFormat::I16 => device.build_output_stream(&cfg, move |out: &mut [i16], _| write_samples(&mut osc, out, channels), |_| {}, None),
        SampleFormat::I32 => device.build_output_stream(&cfg, move |out: &mut [i32], _| write_samples(&mut osc, out, channels), |_| {}, None),
        SampleFormat::F32 => device.build_output_stream(&cfg, move |out: &mut [f32], _| write_samples(&mut osc, out, channels), |_| {}, None),
        SampleFormat::F64 => device.build_output_stream(&cfg, move |out: &mut [f64], _| write_samples(&mut osc, out, channels), |_| {}, None),
        SampleFormat::U8 => device.build_output_stream(&cfg, move |out: &mut [u8], _| write_samples(&mut osc, out, channels), |_| {}, None),
        SampleFormat::U16 => device.build_output_stream(&cfg, move |out: &mut [u16], _| write_samples(&mut osc, out, channels), |_| {}, None),
        SampleFormat::U32 => device.build_output_stream(&cfg, move |out: &mut [u32], _| write_samples(&mut osc, out, channels), |_| {}, None),
        _ => panic!("Unsupported sample format: {:?}", config.sample_format()),
    }
    .expect("Failed to build output stream")
}

pub fn play(mut osc: Oscillator, duration_ms: u64) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config().expect("No default config");

    println!("Format: {:?}, Channels: {}, Rate: {}", config.sample_format(), config.channels(), config.sample_rate());

    osc.sample_rate = config.sample_rate();
    let stream = build_stream(&device, &config, osc);
    stream.play().expect("Failed to play");
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
}
