use std::io::{self, Read, Seek, SeekFrom};
use std::fs;
use std::fs::File;
use std::path::Path;

// Function to check if a file is a WAV file
pub fn is_wavbrro_file(file_path: &Path) -> io::Result<bool> {
    // Open the file for reading and read the first 12 bytes (header) of the file
    let mut file = fs::File::open(file_path)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;
    Ok(&header[0..4] == b"WBRO" && &header[8..12] == b"WBRO")
}

pub fn read_wavbrro_file(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.seek(SeekFrom::Start(12))?;
    file.read_to_end(&mut contents)?;
    Ok(contents)
}