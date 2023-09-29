// Import necessary libraries
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

// Function to check if a file is a WAV file
fn is_wav_file(file_path: &Path) -> io::Result<bool> {
    // Open the file for reading
    let mut file = fs::File::open(file_path)?;

    // Read the first 12 bytes (header) of the file
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;

    // Check if the file starts with "RIFF" and ends with "WAVE" in the header
    Ok(header.starts_with(b"RIFF") && &header[8..12] == b"WAVE")
}

// Function to process a WAV file
fn process_wav_file(file_path: &Path) -> io::Result<()> {
    // Handle WAV file processing here (for example, analyzing or converting)
    println!("Processing WAV file: {:?}", file_path);
    Ok(())
}

// Function to process a RAW file
fn process_raw_file(file_path: &Path) -> io::Result<()> {
    // Handle RAW file processing here (for example, decoding or encoding)
    println!("Processing RAW file: {:?}", file_path);
    Ok(())
}

// Function to read and process files in a directory
fn stream_reader(directory_path: &Path) -> io::Result<()> {
    // Iterate through entries (files and subdirectories) in the given directory
    for entry in fs::read_dir(directory_path)? {
        // Unwrap the entry to get the actual entry information
        let entry = entry?;

        // Get the path of the entry (file or directory)
        let file_path = entry.path();

        // Check if the file is a WAV file
        if is_wav_file(&file_path)? {
            // If it's a WAV file, process it using the process_wav_file function
            process_wav_file(&file_path)?;
        } else {
            // If it's not a WAV file, process it as a RAW file using the process_raw_file function
            process_raw_file(&file_path)?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // Define the directory path where files will be processed
    let directory_path = PathBuf::from(""); // Replace with the actual directory path

    // Call the stream_reader function to process files in the directory
    stream_reader(&directory_path)?;

    Ok(())
}
