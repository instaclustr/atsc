// Implement a streaming reader here
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

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

// Import the testing module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_wav_file_with_wav() {
        // Create a temporary file with a valid WAV header
        let temp_file_path = "test.wav";
        let header: [u8; 12] = [82, 73, 70, 70, 4, 0, 0, 0, 87, 65, 86, 69];
        std::fs::write(temp_file_path, &header).expect("Failed to create temporary WAV file");

        // Check if the file is recognized as a WAV file
        let path = Path::new(temp_file_path);
        let result = is_wav_file(&path);

        // Assert that it should be recognized as a WAV file
        assert!(result.is_ok() && result.unwrap() == true);

        // Clean up the temporary file
        std::fs::remove_file(temp_file_path).expect("Failed to remove temporary file");
    }

    #[test]
    fn test_is_wav_file_with_non_wav() {
        // Create a temporary file with a non-WAV header
        let temp_file_path = "test.txt";
        let header: [u8; 12] = [84, 69, 83, 84, 32, 70, 73, 76, 69, 33, 33, 33];
        std::fs::write(temp_file_path, &header).expect("Failed to create temporary non-WAV file");

        // Check if the file is recognized as a WAV file
        let path = Path::new(temp_file_path);
        let result = is_wav_file(&path);

        // Assert that it should not be recognized as a WAV file
        assert!(result.is_ok() && result.unwrap() == false);

        // Clean up the temporary file
        std::fs::remove_file(temp_file_path).expect("Failed to remove temporary file");
    }
}
