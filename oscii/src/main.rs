use clap::Parser;
use oscii::Oscillator;

#[derive(Parser)]
#[command(about = "带限锯齿波振荡器")]
struct Args {
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
    let osc = Oscillator::new(args.frequency, args.volume.clamp(0.0, 1.0));
    oscii::play(osc, args.duration);
}
