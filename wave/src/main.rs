use clap::Parser;
use std::io::Write;
use wave::WavHeader;

/// 生成 WAV 音频文件（正弦波）
#[derive(Parser)]
#[command(name = "wave", about = "生成正弦波 WAV 音频文件")]
struct Args {
    /// 采样率 (Hz)
    #[arg(short = 'r', long, default_value_t = 44100)]
    sample_rate: u32,

    /// 位深，支持 8 / 16 / 32
    #[arg(short = 'b', long, default_value_t = 16)]
    bit_depth: u8,

    /// 正弦波频率 (Hz)
    #[arg(short = 'f', long, default_value_t = 440.0)]
    frequency: f32,

    /// 时长 (秒)
    #[arg(short = 'd', long, default_value_t = 5.0)]
    duration: f32,

    /// 通道数
    #[arg(short = 'c', long, default_value_t = 1)]
    channels: u16,

    /// 输出文件名
    #[arg(short = 'o', long, default_value = "output.wav")]
    output: String,
}

fn main() {
    let args = Args::parse();

    if ![8u8, 16, 32].contains(&args.bit_depth) {
        eprintln!("错误: 位深必须为 8、16 或 32");
        std::process::exit(1);
    }
    if args.channels == 0 {
        eprintln!("错误: 通道数必须大于 0");
        std::process::exit(1);
    }

    let bytes_per_sample = args.bit_depth as usize / 8;
    let num_frames = (args.sample_rate as f32 * args.duration) as usize;
    let data_size = num_frames * args.channels as usize * bytes_per_sample;

    let wav_header = WavHeader {
        riff: wave::RiffHeader {
            id: *b"RIFF",
            size: (36 + data_size) as u32,
            type_: *b"WAVE",
        },
        fmt: wave::FmtChunk {
            id: *b"fmt ",
            size: 16,
            audio_format: 1,
            num_channels: args.channels,
            sample_rate: args.sample_rate,
            byte_rate: args.sample_rate * args.channels as u32 * bytes_per_sample as u32,
            block_align: args.channels * bytes_per_sample as u16,
            bits_per_sample: args.bit_depth as u16,
        },
        data: wave::DataHeader {
            id: *b"data",
            size: data_size as u32,
        },
    };

    let mut file = std::fs::File::create(&args.output).expect("无法创建输出文件");

    // 写入文件头
    let header_bytes = unsafe {
        std::slice::from_raw_parts(
            &wav_header as *const WavHeader as *const u8,
            std::mem::size_of::<WavHeader>(),
        )
    };
    file.write_all(header_bytes).expect("写入文件头失败");

    // 逐帧写入采样数据，每帧对所有通道写相同的值
    let mut buf = [0u8; 4];
    for i in 0..num_frames {
        let t = i as f32 / args.sample_rate as f32;
        let sample_f = (t * args.frequency * 2.0 * std::f32::consts::PI).sin();

        // 将浮点采样转为目标位深的字节，存入 buf 的前 n 字节
        let n = match args.bit_depth {
            8 => {
                // 8-bit WAV 使用无符号整数，静默点为 128
                buf[0] = ((sample_f * 127.0) + 128.0).clamp(0.0, 255.0) as u8;
                1
            }
            16 => {
                let bytes = ((sample_f * i16::MAX as f32) as i16).to_le_bytes();
                buf[..2].copy_from_slice(&bytes);
                2
            }
            32 => {
                let bytes = ((sample_f * i32::MAX as f32) as i32).to_le_bytes();
                buf[..4].copy_from_slice(&bytes);
                4
            }
            _ => unreachable!(),
        };

        for _ in 0..args.channels {
            file.write_all(&buf[..n]).expect("写入采样数据失败");
        }
    }

    println!(
        "已生成: {}  (采样率: {} Hz | 位深: {} bit | 频率: {} Hz | 时长: {} s | 通道数: {})",
        args.output, args.sample_rate, args.bit_depth, args.frequency, args.duration, args.channels
    );
}
