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

