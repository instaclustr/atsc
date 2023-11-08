use rkyv::{Archive, Deserialize, Serialize};
use std::{io, fmt, result, error};
use std::path::Path;

use crate::read::{is_wavbrro_file, read_wavbrro_file};
use crate::write::write_wavbrro_file;

const MAX_CHUNK_SIZE:usize = 2048;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(
    // This will generate a PartialEq impl between our unarchived and archived
    // types:
    compare(PartialEq),
    // bytecheck can be used to validate your data if you want. To use the safe
    // API, you have to derive CheckBytes for the archived type:
    check_bytes,
)]
// Derives can be passed through to the generated type:
#[archive_attr(derive(Debug))]
pub struct WavBrro {
    // We can infer chunk count from here -> chunk count = ceil(sample_count/MAX_CHUNK_SIZE)
    pub sample_count: u32,
    // Bitdepth indicates the type of samples that is contained in the file
    // 0 -> u8, 1 -> i16, 2 -> i32, 3 -> i64, 4 -> f32, 5 -> f64
    // At version 0.1 only f64 is allowed, so any data will be converted to f64 and all data be output as f64
    pub bitdepth: u8,
    // Fixed size, of 2048 per chunk (Except last segment)
    pub chunks: Vec<Vec<f64>>,
}

impl Default for WavBrro {
    fn default() -> Self {
        Self::new()
    }
}

impl WavBrro {
    pub fn new() -> WavBrro {
        WavBrro {
            // It will duplicate with the header, but this allows to double check if the header matches.
            sample_count: 0,
            // Default to f64
            bitdepth: 5,
            chunks: Vec::new()
        }
    }

    fn is_chunk_full(&self) -> bool {
        match self.chunks.last() {
            Some(c) => { c.len() >= MAX_CHUNK_SIZE },
            None => { true }
        } 
    }

    fn create_chunk(&mut self) {
        // If I'm creating a chunk, I should probably flush the file?
        self.chunks.push(Vec::with_capacity(MAX_CHUNK_SIZE));
    }

    // Receives a slice of f64 and writes in it's internal structure
    fn from_slice(data: &[f64]) -> Self {
        let sample_count = data.len();
        WavBrro {
                sample_count: sample_count as u32,
                bitdepth: 5,
                chunks: data.chunks(MAX_CHUNK_SIZE).map(|s| s.into()).collect()
            }
    }

    pub fn add_sample(&mut self, sample: f64) {
        if self.is_chunk_full() { self.create_chunk() }
        self.chunks.last_mut().unwrap().push(sample);
        self.sample_count += 1;
    }

    // This should be generic, but first implementation is going to be Vec f64
    // This consumes self!
    pub fn get_samples(self) -> Vec<f64> {
        self.chunks.into_iter()
                   .flatten()
                   .collect::<Vec<f64>>()
    }

    // This should be generic, but first implementation is going to be Vec f64
    // TODO: This will panic left and right, make it right
    pub fn from_file(file_path: &Path) -> Result<Vec<f64>, Error> {
        // Check if the header is correct
        if !is_wavbrro_file(file_path) {return Err(Error::FormatError);};
        let bytes = read_wavbrro_file(file_path)?;
        let obj = WavBrro::from_bytes(&bytes);
        Ok(obj.get_samples())
    }

    // TODO: This will panic left and right, make it right
    pub fn to_file_with_data(file_path: &Path, data: &[f64]) {
        let wb = WavBrro::from_slice(data);
        let bytes = wb.to_bytes();
        write_wavbrro_file(file_path, &bytes);
    }

    // TODO: This will panic left and right, make it right
    pub fn to_file(&self, file_path: &Path) {
        let bytes = self.to_bytes();
        write_wavbrro_file(file_path, &bytes);
    }

    pub fn to_bytes(&self) -> rkyv::AlignedVec {
        rkyv::to_bytes::<_, 1024>(self).expect("Failed to serialize data!")
    }

    pub fn from_bytes(bytes: &[u8] ) -> Self {
        rkyv::from_bytes::<WavBrro>(bytes).expect("Failed to deserialize data!")
    }

}

// Error class is based on https://codeberg.org/ruuda/hound/src/branch/master given the similarities 
// between the formats (WAV and WAVBRRO).
#[derive(Debug)]
pub enum Error {
    /// An IO error occured in the underlying reader or writer.
    IoError(io::Error),
    /// It's not WAVBRRO
    FormatError,
    /// The sample has more bits than the destination type.
    ///
    /// When iterating using the `samples` iterator, this means that the
    /// destination type (produced by the iterator) is not wide enough to hold
    /// the sample. When writing, this means that the sample cannot be written,
    /// because it requires more bits than the bits per sample specified.
    TooWide,
    /// The Sample format is not supported.
    Unsupported,
    /// The sample format is different than the destination format.
    ///
    /// When iterating using the `samples` iterator, this means the destination
    /// type (produced by the iterator) has a different sample format than the
    /// samples in the wav file.
    ///
    /// For example, this will occur if the user attempts to produce `i32`
    /// samples (which have a `SampleFormat::Int`) from a wav file that
    /// contains floating point data (`SampleFormat::Float`).
    InvalidSampleFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Error::IoError(ref err) => err.fmt(formatter),
            Error::FormatError => {
                formatter.write_str("Wrong WAVBRRO file!")
            }
            Error::TooWide => {
                formatter.write_str("The sample has more bits than the destination type.")
            }
            Error::Unsupported => {
                formatter.write_str("The WAVBRRO format of the file is not supported.")
            }
            Error::InvalidSampleFormat => {
                formatter.write_str("The sample format differs from the destination format.")
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            // TODO: I don't know if this is actually the right way to do!
            Error::IoError(ref _err) => "IO Error",
            Error::TooWide => "the sample has more bits than the destination type",
            Error::Unsupported => "the wave format of the file is not supported",
            Error::InvalidSampleFormat => "the sample format differs from the destination format",
            Error::FormatError => "the file is not of the WAVBRRO format",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::IoError(ref err) => Some(err),
            Error::TooWide => None,
            Error::Unsupported => None,
            Error::InvalidSampleFormat => None,
            Error::FormatError => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wavbrro() {
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        assert_eq!(wb.sample_count, 1);
    }

    #[test]
    fn test_serialization() {
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        assert_eq!(wb.to_bytes().as_slice(), &[0, 0, 0, 0, 0, 0, 240, 63, 248, 255, 255, 255, 1, 0, 0, 0, 248, 255, 255, 255, 1, 0, 0, 0, 1, 0, 0, 0, 5, 0, 0, 0]);
    }

    #[test]
    fn test_deserialization() {
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        wb.add_sample(2.0);
        wb.add_sample(3.0);
        let bytes = wb.to_bytes();
        let wb2 = WavBrro::from_bytes(bytes.as_slice());
        assert_eq!(wb, wb2);
    }

    #[test]
    fn test_write_wavbrro() {
        let path = Path::new("test.wbro");
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        wb.add_sample(2.0);
        wb.add_sample(3.0);
        wb.to_file(path);
        let result = is_wavbrro_file(path);
        assert!(result);
        std::fs::remove_file(path).expect("Failed to remove temporary file");
    }

    #[test]
    fn test_read_wavbrro() {
        let path = Path::new("test.wbro");
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        wb.add_sample(2.0);
        wb.add_sample(3.0);
        wb.to_file(path);
        let data = WavBrro::from_file(path);
        assert_eq!(data.unwrap(), [1.0, 2.0, 3.0]);
        std::fs::remove_file(path).expect("Failed to remove temporary file");
    }
}