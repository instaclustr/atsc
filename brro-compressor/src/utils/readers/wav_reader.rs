// Implement a streaming reader here
use std::fs;
use std::io::{self, Error, Read};
use std::path::{Path, PathBuf};
use log::debug;
use regex::Regex;
use types::metric_tag::MetricTag;
use crate::types;


// Function to check if a file is a WAV file
fn is_wav_file(file_path: &Path) -> io::Result<bool> {
    // Open the file for reading and read the first 12 bytes (header) of the file
    let mut file = fs::File::open(file_path)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;

    // Check if the file starts with "RIFF" and ends with "WAVE" in the header
    Ok(header.starts_with(b"RIFF") && &header[8..12] == b"WAVE")
}

// Function to process a WAV file
fn process_wav_file(file_path: &Path) -> io::Result<(Vec<f64>, MetricTag)> {
    let full_path_str = file_path.to_str().unwrap_or("");
    debug!("File: {} ,", full_path_str);
    let wav_data = read_metrics_from_wav(full_path_str);
    debug!("Data Len: {}", wav_data.len());
    // Depending on Metric Tag, apply a transformation
    let tag = tag_metric(full_path_str);
    Ok((wav_data, tag))
}

// Function to process a RAW file
fn process_raw_file(file_path: &Path) -> io::Result<(Vec<f64>, MetricTag)> {
    todo!("Handle RAW file processing here (for example, decoding or encoding): {file_path:?}");
}

pub struct WavFile {
    pub contents: Vec<f64>,
    pub tag: MetricTag,
    pub original_path: PathBuf,
}

/// Read a file by chunks and processes the chunks
pub fn process_by_chunk(file_path: &Path) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::open(file_path)?;

    let mut list_of_chunks = Vec::new();
    // 64KB at a time, assuming 64Bit samples, ~1024 samples.
    let chunk_size = 0x10000;

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let n = file.by_ref().take(chunk_size as u64).read_to_end(&mut chunk)?;
        if n == 0 { break; }
        list_of_chunks.push(chunk);
        if n < chunk_size { break; }
    }
    Ok(())
}

// Function to read and process files in a directory
pub fn dir_reader(directory_path: &Path) -> io::Result<Vec<WavFile>> {
    let mut files = vec!();
    for entry in fs::read_dir(directory_path)? {
        let file_path = entry?.path();

        let (contents, tag) = read_file(&file_path)?;
        files.push(WavFile {
            contents,
            tag,
            original_path: file_path,
        })
    }
    Ok(files)
}

pub fn read_file(file_path: &Path) -> Result<(Vec<f64>, MetricTag), Error> {
    if is_wav_file(file_path)? {
        // If it's a WAV file, process it using the process_wav_file function
        process_wav_file(file_path)
    } else {
        // If it's not a WAV file, process it as a RAW file using the process_raw_file function
        process_raw_file(file_path)
    }
}
/*
Reads a WAV file, checks the channels and the information contained there. From that
information takes a decision on the best channel, block size and bitrate for the BRRO
encoders.
*/
fn tag_metric(filename: &str) -> MetricTag {
    // Should sort this by the probability of each tag, so the ones that are more common are dealt first
    // If it says percent_ or _utilization
    let mut regex = Regex::new(r"(?m)percent_|_utilization").unwrap();
    if regex.captures(filename).is_some() {
        // 2 significant digits resolution (Linux resolution)
        return MetricTag::Percent(100);
    }
    // if it says _client_request
    regex = Regex::new(r"(?m)_client_request").unwrap();
    if regex.captures(filename).is_some() {
        // Fractional requests are nothing but an averaging artifact
        return MetricTag::NotFloat;
    }
    // if it says _seconds
    regex = Regex::new(r"(?m)_seconds").unwrap();
    if regex.captures(filename).is_some() {
        // 1 micro second resolution
        return MetricTag::Duration(1_000_000);
    }
    // if it says _seconds
    regex = Regex::new(r"(?m)_networkindelta|_networkoutdelta|_heapmemoryused_").unwrap();
    if regex.captures(filename).is_some() {
        return MetricTag::QuasiRandom;
    }
    MetricTag::Other
}

fn read_metrics_from_wav(filename: &str) -> Vec<f64> {
    let r_reader = hound::WavReader::open(filename);
    let mut reader = match r_reader {
        Ok(reader) => reader,
        Err(_err) => { return Vec::new(); }
    };
    let num_channels = reader.spec().channels as usize;

    let mut raw_data: Vec<f64> = Vec::new();
    let mut u64_holder: [u16; 4] = [0, 0, 0, 0];

    // Iterate over the samples and channels and push each sample to the vector
    let mut current_channel: usize = 0;
    for sample in reader.samples::<i16>() {
        u64_holder[current_channel] = sample.unwrap() as u16;
        current_channel += 1;
        if current_channel == num_channels {
            raw_data.push(join_u16_into_f64(u64_holder));
            current_channel = 0;
        }
    }
    raw_data
}

fn join_u16_into_f64(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64) |
        ((bits[1] as u64) << 16) |
        ((bits[2] as u64) << 32) |
        ((bits[3] as u64) << 48);


    f64::from_bits(u64_bits)
}

// Import the testing module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_wav_file_with_wav() {
        // Create a temporary file with a valid WAV header
        let temp_file_path = "test.wav";
        let header: [u8; 12] = [82, 73, 70, 70, 4, 0, 0, 0, 87, 65, 86, 69];
        std::fs::write(temp_file_path, header).expect("Failed to create temporary WAV file");

        // Check if the file is recognized as a WAV file
        let path = Path::new(temp_file_path);
        let result = is_wav_file(path);

        // Assert that it should be recognized as a WAV file
        assert!(result.is_ok() && result.unwrap());

        // Clean up the temporary file
        std::fs::remove_file(temp_file_path).expect("Failed to remove temporary file");
    }

    #[test]
    fn test_is_wav_file_with_non_wav() {
        // Create a temporary file with a non-WAV header
        let temp_file_path = "test.txt";
        let header: [u8; 12] = [84, 69, 83, 84, 32, 70, 73, 76, 69, 33, 33, 33];
        std::fs::write(temp_file_path, header).expect("Failed to create temporary non-WAV file");

        // Check if the file is recognized as a WAV file
        let path = Path::new(temp_file_path);
        let result = is_wav_file(path);

        // Assert that it should not be recognized as a WAV file
        assert!(result.is_ok() && !result.unwrap());

        // Clean up the temporary file
        std::fs::remove_file(temp_file_path).expect("Failed to remove temporary file");
    }
}