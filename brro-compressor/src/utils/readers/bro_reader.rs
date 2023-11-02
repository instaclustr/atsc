// Implement a streaming reader here
use std::fs;
use std::fs::File;
use std::io::{self, Error, Read};
use std::path::Path;

// Function to process a BRRO file
fn process_bro_file(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn read_file(file_path: &Path) -> Result<Option<Vec<u8>>, Error> {
    if is_bro_file(file_path)? {
        // If it's a WAV file, process it using the process_wav_file function
        Ok(Some(process_bro_file(file_path)?))
    } else {
        Ok(None)
    }
}
fn is_bro_file(file_path: &Path) -> io::Result<bool> {
    // Open the file for reading and read the first 12 bytes (header) of the file
    let mut file = fs::File::open(file_path)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;
    // Check if the file starts with "BRRO"
    Ok(header.starts_with(b"BRRO"))
}

/// Read a file by chunks and processes the chunks
pub fn process_by_chunk(file_path: &Path) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::open(file_path)?;

    let mut list_of_chunks = Vec::new();
    // 64KB at a time, assuming 64Bit samples, ~1024 samples.
    let chunk_size = 0x10000;

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let n = file
            .by_ref()
            .take(chunk_size as u64)
            .read_to_end(&mut chunk)?;
        if n == 0 {
            break;
        }
        list_of_chunks.push(chunk);
        if n < chunk_size {
            break;
        }
    }
    Ok(())
}
