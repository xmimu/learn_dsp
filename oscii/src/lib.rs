use cpal::{
    FromSample, SampleFormat,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

/// 单个波表的采样点数
const TBL_SIZE: usize = 4096;
/// 第一张表对应的基频
const MIN_FREQ: f32 = 20.0;

/// 一张带限波表，适用于频率 < `max_freq` 的振荡
pub struct WaveTable {
    max_freq: f32,
    buf: Vec<f32>,
}

impl WaveTable {
    /// 给定相位 [0,1) 返回插值后的采样值
    fn eval(&self, phase: f32) -> f32 {
        let fr = phase * TBL_SIZE as f32;
        let i = fr as usize;
        let frac = fr - i as f32;

        let x0 = self.buf[i % TBL_SIZE];
        let x1 = self.buf[(i + 1) % TBL_SIZE];

        (1.0 - frac) * x0 + frac * x1
    }
}

/// 锯齿波傅里叶系数 b[k] = (-1)^(k+1) / k
fn coeff(k: usize) -> f32 {
    if k % 2 == 0 {
        -1.0 / k as f32
    } else {
        1.0 / k as f32
    }
}

/// 生成基频为 `freq`、不超过 `nyquist` 的带限锯齿波表，幅度归一化到 [-1,1]
fn gen_table(freq: f32, nyquist: f32) -> WaveTable {
    let step = 2.0 * std::f32::consts::PI / TBL_SIZE as f32;
    let mut buf = vec![0.0f32; TBL_SIZE];
    let mut max = 0.0f32;

    for i in 0..TBL_SIZE {
        let phi = i as f32 * step;
        let mut sample = 0.0f32;
        let mut k = 1usize;
        while (k as f32) * freq < nyquist {
            sample += coeff(k) * (k as f32 * phi).sin();
            k += 1;
        }
        buf[i] = sample;
        let abs = sample.abs();
        if abs > max {
            max = abs;
        }
    }

    if max > 0.0 {
        for s in &mut buf {
            *s /= max;
        }
    }

    WaveTable { max_freq: freq, buf }
}

/// 按倍频程构造一组带限锯齿波表
fn build_tables(sample_rate: u32) -> Vec<WaveTable> {
    let nyquist = sample_rate as f32 / 2.0;
    let n = (nyquist / MIN_FREQ).log2() as usize + 1;
    let mut tables = Vec::with_capacity(n);
    let mut freq = MIN_FREQ;
    for _ in 0..n {
        tables.push(gen_table(freq, nyquist));
        freq *= 2.0;
    }
    tables
}

pub struct Oscillator {
    frequency: f32,
    pub(crate) sample_rate: u32,
    phase: f32,
    volume: f32,
    tables: Vec<WaveTable>,
}

impl Oscillator {
    pub fn new(frequency: f32, volume: f32) -> Self {
        Self {
            frequency,
            sample_rate: 0,
            phase: 0.0,
            volume,
            tables: Vec::new(),
        }
    }

    fn pick_table(&self, freq: f32) -> &WaveTable {
        for t in &self.tables {
            if freq < t.max_freq {
                return t;
            }
        }
        self.tables.last().expect("no wavetables built")
    }

    fn next_sample(&mut self) -> f32 {
        let sample = self.pick_table(self.frequency).eval(self.phase);
        self.phase = (self.phase + self.frequency / self.sample_rate as f32) % 1.0;
        sample * self.volume
    }
}

fn write_samples<T: cpal::Sample + FromSample<f32>>(
    osc: &mut Oscillator,
    output: &mut [T],
    channels: usize,
) {
    for frame in output.chunks_mut(channels) {
        let s = T::from_sample(osc.next_sample());
        for sample in frame.iter_mut() {
            *sample = s;
        }
    }
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    mut osc: Oscillator,
) -> cpal::Stream {
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

    println!(
        "Format: {:?}, Channels: {}, Rate: {}",
        config.sample_format(),
        config.channels(),
        config.sample_rate()
    );

    osc.sample_rate = config.sample_rate();
    osc.tables = build_tables(osc.sample_rate);
    println!("Bandlimited saw tables: {}", osc.tables.len());

    let stream = build_stream(&device, &config, osc);
    stream.play().expect("Failed to play");
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tables_count() {
        let tables = build_tables(48000);
        // log2(24000/20) ≈ 10.23 → 11 张表
        assert_eq!(tables.len(), 11);
        assert_eq!(tables[0].max_freq, MIN_FREQ);
        assert_eq!(tables[1].max_freq, MIN_FREQ * 2.0);
    }

    #[test]
    fn test_table_normalized() {
        let t = gen_table(MIN_FREQ, 24000.0);
        let max = t.buf.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
        assert!((max - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_eval_interpolation() {
        // 用一张线性表验证插值
        let buf: Vec<f32> = (0..TBL_SIZE).map(|i| i as f32).collect();
        let t = WaveTable { max_freq: 0.0, buf };
        let phase = 100.5 / TBL_SIZE as f32;
        let v = t.eval(phase);
        assert!((v - 100.5).abs() < 1e-3);
    }
}
