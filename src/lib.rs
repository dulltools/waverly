//! A library for reading and writing WAV files.
//!
//! This library is meant to provide access to all data within a WAV file,
//! including FACT and PEAK chunks and extensible version of format chunks.
//!
//! It also supports `no_std`.
//!
//! # Usage
//!
//! First, add this to your `Cargo.toml`
//!
//! ```toml
//! [dependencies]
//! waverly = "0.1"
//! ```
//!
//! Next:
//!
//! ```
//! use std::fs::File;
//! use waverly::Wave;
//! use std::io::Cursor;
//! fn main() -> Result<(), waverly::WaverlyError> {
//!     let file = File::open("./meta/16bit-2ch-float-peak.wav")?;
//!     let wave: Wave = Wave::from_reader(file)?;
//!
//!     let mut virt_file = Cursor::new(Vec::new());
//!     wave.write(&mut virt_file)?;
//!     Ok(())
//! }
//! ```

#![cfg_attr(not(feature="std"), no_std)]

#[cfg(not(feature="std"))]
extern crate alloc;

#[cfg(not(feature="std"))]
use alloc::vec::Vec;

use binrw::{binrw, until_exclusive, io, BinRead, BinWrite};
use binrw::io::{Read, Write, Seek, ErrorKind};

pub type Result<T> = core::result::Result<T, WaverlyError>;

#[derive(Debug)]
pub enum WaverlyError {
    IoError(io::Error),
    ParseError(binrw::Error),
}

impl From<io::Error> for WaverlyError {
    fn from(error: io::Error) -> Self {
        WaverlyError::IoError(error)
    }
}
impl From<binrw::Error> for WaverlyError {
    fn from(error: binrw::Error) -> Self {
        WaverlyError::ParseError(error)
    }
}

#[binrw]
#[derive(Debug)]
struct MyFile {
    #[br(parse_with = until_exclusive(|byte| byte == &Chunk::Unhandled))]
    chunks: Vec<Chunk>,
}

#[binrw]
#[derive(Debug, PartialEq)]
enum Chunk {
    Riff(RiffChunk),
    Format(FormatChunk),
    Fact(FactChunk),
    Peak(PeakChunk),
    Data(DataChunk),
    // eof
    #[brw(magic = b"")]
    Unhandled,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Debug, PartialEq)]
pub enum WaveFormat {
    Pcm = 0x01,
    IeeeFloat = 0x03,
    /// 8-bit ITU-T G.711 A-law
    Alaw = 0x06,
    /// 8-bit ITU-T G.711 µ-law
    Mulaw = 0x07,
    Extensible = 0x08,
}

#[binrw]
#[brw(magic = b"WAVEfmt ")]
#[derive(Debug, PartialEq)]
pub struct FormatChunk {
    #[br(little)]
    pub format_chunk_size: u32,
    #[br(little)]
    pub audio_format: WaveFormat,
    #[br(little)]
    pub num_channels: u16,
    #[br(little)]
    pub sample_rate: u32,
    /// Average number of bytes per second at which the data should be transferred
    #[br(little)]
    pub byte_rate: u32,
    /// The block alignment (in bytes) of the waveform data. Playback
    /// software needs to process a multiple of wBlockAlign bytes of data at
    /// a time, so the value of wBlockAlign can be used for buffer
    /// alignment. 
    #[br(little)]
    pub block_align: u16,
    #[br(little)]
    pub bits_per_sample: u16,
    #[br(little, if(audio_format == WaveFormat::Extensible))]
    pub extensible: Option<ExtensibleFormat>,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ExtensibleFormat {
    #[br(little)]
    pub size: u16,
    #[br(little)]
    pub valid_bits_per_sample: u16,
    #[br(little)]
    pub channel_mask: u32,
    #[br(little)]
    pub sub_format_guid: [u8; 16],
}

#[binrw]
#[brw(magic = b"fact")]
#[derive(Debug, PartialEq)]
pub struct FactChunk {
    #[br(little)]
    pub size: u32,
    #[br(little)]
    pub data: u32,
}

#[binrw]
#[brw(magic = b"data")]
#[derive(Debug, PartialEq)]
pub struct DataChunk {
    #[br(little)]
    pub size: u32,
    #[br(count = size)]
    pub data: Vec<u8>,
}

/// Indicates the peak amplitude of the soundfile
#[binrw]
#[brw(magic = b"PEAK")]
#[derive(Debug, PartialEq)]
pub struct PeakChunk {
    #[br(little)]
    pub size: u32,
    #[br(little)]
    pub version: u32,
    /// Unix epoch. This is used to see if the date of the peak data 
    /// matches the modification date of the file. If not, the file
    /// should be rescanned for new peak data.
    #[br(little)]
    pub timestamp: u32,
    /// PositionPeak for each channel, in the same order as the samples
    /// are interleaved.
    #[br(count = 2)]
    pub peaks: Vec<Peak>,
}

/// Amplitude peak
#[binrw]
#[derive(Debug, PartialEq)]
pub struct Peak {
    #[br(little)]
    pub value: f32,
    /// The sample frame number at which the peak occurs. Note
    /// that the unit for position are sample frames, not sample points nor
    /// bytes.
    #[br(little)]
    pub position: u32,
}

#[binrw]
#[brw(magic = b"RIFF")]
#[derive(Debug, PartialEq)]
struct RiffChunk {
    #[br(little)]
    chunk_size: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct Wave {
    riff: RiffChunk,
    pub format: FormatChunk,
    pub data: DataChunk,
    pub fact: Option<FactChunk>,
    pub peak: Option<PeakChunk>,
}

impl Wave {
    pub fn from_reader<T: Seek + Read>(mut reader: T) -> Result<Wave> {
        let my_file: MyFile = MyFile::read(&mut reader)?;

        let mut riff = None;
        let mut format = None;
        let mut data = None;
        let mut fact = None;
        let mut peak = None;

        for chunk in my_file.chunks {
            match chunk {
                Chunk::Riff(chunk) => riff = Some(chunk),
                Chunk::Data(chunk) => data = Some(chunk),
                Chunk::Format(chunk) => format = Some(chunk),
                Chunk::Fact(chunk) => fact = Some(chunk),
                Chunk::Peak(chunk) => peak = Some(chunk),
                Chunk::Unhandled => (),
            }
        }

        if riff == None {
            return Err(io::Error::new(ErrorKind::InvalidInput, "RIFF chunk was not found in file.").into());
        }

        if data == None {
            return Err(io::Error::new(ErrorKind::InvalidInput, "DATA chunk was not found in file.").into());
        }

        if format == None {
            return Err(io::Error::new(ErrorKind::InvalidInput, "FORMAT chunk was not found in file.").into());
        }


        Ok(Wave {
            riff: riff.unwrap(),
            data: data.unwrap(),
            format: format.unwrap(),
            fact,
            peak,
        })
    }

    pub fn write<T: Seek + Write>(self, mut writer: T) -> Result<()> {
        self.write_to(&mut writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::io::Cursor;
    use std::fs;
    use std::fs::File;

    #[test]
    fn it_pulls_format_chunk_correctly() -> Result<()> {
        let file = File::open("./meta/16bit-2ch-float-peak.wav")?;
        let wave: Wave = Wave::from_reader(file)?;

        let f = &wave.format;
        assert_eq!(f.sample_rate, 44100);
        assert_eq!(f.bits_per_sample, 64);
        assert_eq!(f.num_channels, 2);
        assert_eq!(f.audio_format, WaveFormat::IeeeFloat);

        let block_align = f.num_channels * f.bits_per_sample / 8;
        let byte_rate = f.sample_rate * block_align as u32;
        assert_eq!(f.byte_rate, byte_rate);
        assert_eq!(f.byte_rate, 705600);
        assert_eq!(f.block_align, block_align);
        assert_eq!(f.block_align, 16);
        assert_eq!(f.extensible, None);

        Ok(())
    }

    #[test]
    fn it_writes_data_correctly() -> Result<()> {
        let filename = "./meta/16bit-2ch-float-peak.wav";
        let file = File::open(filename)?;
        let wave: Wave = Wave::from_reader(file)?;
        let metadata = fs::metadata(filename)?;

        let mut virt_file = Cursor::new(Vec::new());
        wave.write(&mut virt_file)?;
        let buf = virt_file.into_inner();
        assert_eq!(buf.len(), metadata.len() as usize);
        assert_ne!(buf.len(), 0);
        let buf_iter = buf.into_iter();
        let riff_magic: Vec<u8> = buf_iter.take(4).collect();
        assert_eq!([82, 73, 70, 70], riff_magic[..]);

        Ok(())
    }
}
