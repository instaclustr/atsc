// Implement a streaming reader here
use std::fs;
use std::fs::File;
use std::io::{self, Error, Read};
use std::path::{Path, PathBuf};

// Function to process a WAV file
fn process_bro_file(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub struct BroFile {
    pub contents: Vec<u8>,
    pub original_path: PathBuf,
}

// Function to read and process files in a directory
pub fn dir_reader(directory_path: &Path) -> io::Result<Vec<BroFile>> {
    let mut files = vec![];

    // Iterate through entries (files and subdirectories) in the given directory
    for entry in fs::read_dir(directory_path)? {
        let file_path = entry?.path();

        files.push(BroFile {
            contents: read_file(&file_path)?,
            original_path: file_path,
        })
    }

    Ok(files)
}

pub fn read_file(file_path: &Path) -> Result<Vec<u8>, Error> {
    if is_bro_file(file_path)? {
        // If it's a WAV file, process it using the process_wav_file function
        Ok(process_bro_file(file_path)?)
    } else {
        Err(Error::new(io::ErrorKind::Other, "File is not a bro file"))
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
