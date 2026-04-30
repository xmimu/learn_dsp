use std::io::Write;
use clap::Parser;

pub type Fourcc = [u8; 4];

#[repr(C, packed)]
pub struct RiffHeader {
    pub id: Fourcc,
    pub size: u32,
    pub type_: Fourcc,
}

#[repr(C, packed)]
pub struct FmtChunk {
    pub id: Fourcc,
    pub size: u32,
    pub audio_format: u16,
    pub num_channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

#[repr(C, packed)]
pub struct DataHeader {
    pub id: Fourcc,
    pub size: u32,
}

#[repr(C, packed)]
pub struct WavHeader {
    pub riff: RiffHeader,
    pub fmt: FmtChunk,
    pub data: DataHeader,
}

impl WavHeader {
    pub fn new(channels: u16, sample_rate: u32, bit_depth: u8, num_frames: usize) -> Self {
        let bytes_per_sample = bit_depth as usize / 8;
        let data_size = num_frames * channels as usize * bytes_per_sample;
        WavHeader {
            riff: RiffHeader {
                id: *b"RIFF",
                size: (36 + data_size) as u32,
                type_: *b"WAVE",
            },
            fmt: FmtChunk {
                id: *b"fmt ",
                size: 16,
                audio_format: 1,
                num_channels: channels,
                sample_rate,
                byte_rate: sample_rate * channels as u32 * bytes_per_sample as u32,
                block_align: channels * bytes_per_sample as u16,
                bits_per_sample: bit_depth as u16,
            },
            data: DataHeader {
                id: *b"data",
                size: data_size as u32,
            },
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        // Safety: WavHeader 为 #[repr(C, packed)]，布局固定，所有位模式对 u8 均合法。
        unsafe {
            std::slice::from_raw_parts(
                self as *const WavHeader as *const u8,
                std::mem::size_of::<WavHeader>(),
            )
        }
    }
}

/// 生成 WAV 音频文件（正弦波）
#[derive(Parser)]
#[command(name = "wave", about = "生成正弦波 WAV 音频文件")]
pub struct Args {
    /// 采样率 (Hz)
    #[arg(short = 'r', long, default_value_t = 44100)]
    pub sample_rate: u32,

    /// 位深，支持 8 / 16 / 32
    #[arg(short = 'b', long, default_value_t = 16)]
    pub bit_depth: u8,

    /// 正弦波频率 (Hz)
    #[arg(short = 'f', long, default_value_t = 440.0)]
    pub frequency: f32,

    /// 时长 (秒)
    #[arg(short = 'd', long, default_value_t = 5.0)]
    pub duration: f32,

    /// 通道数
    #[arg(short = 'c', long, default_value_t = 1)]
    pub channels: u16,

    /// 输出文件名
    #[arg(short = 'o', long, default_value = "output.wav")]
    pub output: String,
}

pub fn validate_args(args: &Args) {
    if ![8u8, 16, 32].contains(&args.bit_depth) {
        eprintln!("错误: 位深必须为 8、16 或 32");
        std::process::exit(1);
    }
    if args.channels == 0 {
        eprintln!("错误: 通道数必须大于 0");
        std::process::exit(1);
    }
}

/// 将浮点采样（[-1.0, 1.0]）编码为目标位深的小端字节，写入 buf 前 n 字节并返回 n。
fn encode_sample(sample_f: f32, bit_depth: u8, buf: &mut [u8; 4]) -> usize {
    match bit_depth {
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
    }
}

pub fn generate_wav(args: &Args) -> std::io::Result<()> {
    let num_frames = (args.sample_rate as f32 * args.duration) as usize;
    let header = WavHeader::new(args.channels, args.sample_rate, args.bit_depth, num_frames);

    let mut file = std::fs::File::create(&args.output)?;
    file.write_all(header.as_bytes())?;

    // 逐帧写入采样数据，每帧对所有通道写相同的值
    let mut buf = [0u8; 4];
    for i in 0..num_frames {
        let t = i as f32 / args.sample_rate as f32;
        let sample_f = (t * args.frequency * 2.0 * std::f32::consts::PI).sin();
        let n = encode_sample(sample_f, args.bit_depth, &mut buf);
        for _ in 0..args.channels {
            file.write_all(&buf[..n])?;
        }
    }

    Ok(())
}

