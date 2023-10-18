// ... [previous imports and struct definitions]
// Implement a streaming reader here
use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use regex::Regex;
use types::metric_tag::MetricTag;
use crate::types;

pub struct StreamReader {
    path: PathBuf,
    pub contents_f: Vec<Vec<f64>>,
    pub contents_u: Vec<Vec<u8>>,
    pub names: Vec<String>,
    pub tags: Vec<MetricTag>,
}

impl StreamReader {
    // Constructor for StreamReader with directory path
    pub fn from_directory(path: PathBuf) -> Self {
        let mut reader = Self {
            path: path.clone(),
            contents_f: Vec::new(),
            contents_u: Vec::new(),
            names: Vec::new(),
            tags: Vec::new(),
        };

        reader.read_and_process_dir(&path).expect("TODO: panic message");
        reader
    }

    // Constructor for StreamReader with a single file path
    pub fn from_file(file_path: PathBuf) -> Self {
        let mut reader = Self {
            path: file_path.clone(),
            contents_f: Vec::new(),
            contents_u: Vec::new(),
            names: Vec::new(),
            tags: Vec::new(),
        };

        // Immediately process the file upon creation
        if let Err(err) = reader.read_and_process_file(&file_path) {
            eprintln!("Error processing the file: {}", err);
        }

        reader
    }

    // ... [previous methods]

    // Method to read and process an individual file
    fn read_and_process_file(&mut self, file_path: &Path) -> io::Result<()> {
        // Store the name
        if let Some(filename) = file_path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str == ".DS_Store" {
                    return Ok(()); // Skip the file
                }
                self.names.push(filename_str.to_string());
            }
        }

        // Process and store the file content
        if is_wav_file(file_path)? {
            let filename_str = file_path.to_str().unwrap_or("");
            let file_content = read_metrics_from_wav(filename_str);
            self.contents_f.push(file_content);

            // Store the metric tag
            let tag = tag_metric(filename_str);
            self.tags.push(tag);
        } else if is_bro_file(file_path)? {
            let mut file = File::open(file_path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            self.contents_u.push(contents);
        }


        Ok(())
    }
    fn read_and_process_dir(&mut self, dir_path: &Path) -> io::Result<()> {
        // Iterate over the entries in the directory
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            self.read_and_process_file(&path)?;
        }

        Ok(())
    }
}

// Function to check if a file is a WAV file
fn is_wav_file(file_path: &Path) -> io::Result<bool> {
    // Open the file for reading and read the first 12 bytes (header) of the file
    let mut file = fs::File::open(file_path)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;

    // Check if the file starts with "RIFF" and ends with "WAVE" in the header
    Ok(header.starts_with(b"RIFF") && &header[8..12] == b"WAVE")
}

fn is_bro_file(file_path: &Path) -> io::Result<bool> {
    // Open the file for reading and read the first 12 bytes (header) of the file
    let mut file = fs::File::open(file_path)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;
    // Check if the file starts with "BRRO"
    Ok(header.starts_with(b"BRRO"))
}

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
// Use for directory processing
// let mut reader = StreamReader::from_directory(PathBuf::from("/path/to/directory"));
// reader.process_directory().unwrap();

// Use for single file processing
// let reader = StreamReader::from_file(PathBuf::from("/path/to/file.wav"));
