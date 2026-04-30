use clap::Parser;
use wave::{Args, generate_wav, validate_args};

fn main() {
    let args = Args::parse();
    validate_args(&args);

    if let Err(e) = generate_wav(&args) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }

    println!(
        "已生成: {}  (采样率: {} Hz | 位深: {} bit | 频率: {} Hz | 时长: {} s | 通道数: {})",
        args.output, args.sample_rate, args.bit_depth, args.frequency, args.duration, args.channels
    );
}
