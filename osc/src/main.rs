use clap::Parser;
use osc::{Oscillator, WaveForm};

#[derive(Parser)]
#[command(about = "简易振荡器")]
struct Args {
    /// 波形类型（0=正弦 1=方波 2=三角 3=锯齿）
    #[arg(short, long, default_value_t = 0)]
    waveform: u8,
    /// 时长（毫秒）
    #[arg(short, long, default_value_t = 1000)]
    duration: u64,
    /// 频率（Hz）
    #[arg(short, long, default_value_t = 440.0)]
    frequency: f32,
    /// 音量（0.0 - 1.0）
    #[arg(short = 'v', long, default_value_t = 0.1)]
    volume: f32,
}

fn main() {
    let args = Args::parse();

    let waveform = match args.waveform {
        0 => WaveForm::Sine,
        1 => WaveForm::Square,
        2 => WaveForm::Triangle,
        3 => WaveForm::Sawtooth,
        _ => panic!("无效波形，可选: 0=正弦 1=方波 2=三角 3=锯齿"),
    };

    let osc = Oscillator::new(
        args.frequency,
        waveform,
        args.volume.clamp(0.0, 1.0),
    );

    osc::play(osc, args.duration);
}
