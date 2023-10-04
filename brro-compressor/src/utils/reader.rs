// Implement a streaming reader here
use std::fs;
use std::io::{self, Read};
use std::path::Path;
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
    // Depending on Metric Tag, apply a transformation
    let tag = tag_metric(full_path_str);
    return Ok((wav_data, tag));
}

// Function to process a RAW file
fn process_raw_file(file_path: &Path) -> io::Result<()> {
    // Handle RAW file processing here (for example, decoding or encoding)
    println!("Processing RAW file: {:?}", file_path);
    Ok(())
}

// Function to read and process files in a directory
pub fn stream_reader(directory_path: &Path) -> io::Result<(Vec<(Vec<f64>, MetricTag)>, Vec<String>)> {
    let mut results: Vec<(Vec<f64>, MetricTag)> = Vec::new();

    let mut filenames: Vec<String> = Vec::new();


    // Iterate through entries (files and subdirectories) in the given directory
    for entry in fs::read_dir(directory_path)? {
        // Unwrap the entry to get the actual entry information
        let entry = entry?;

        // Get the path of the entry (file or directory)
        let file_path = entry.path();

        // Add the filename to the list
        if let Some(filename) = file_path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                filenames.push(filename_str.to_string());
            }
        }

        // Check if the file is a WAV file
        if is_wav_file(&file_path)? {
            // If it's a WAV file, process it using the process_wav_file function
            let wav_result = process_wav_file(&file_path)?;
            results.push(wav_result);
        } else {
            // If it's not a WAV file, process it as a RAW file using the process_raw_file function
            process_raw_file(&file_path)?;
        }
    }
    Ok((results, filenames))
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