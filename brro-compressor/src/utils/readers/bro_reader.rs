// Implement a streaming reader here
use std::fs;
use std::fs::File;
use std::io::{self, Error, Read};
use std::path::Path;

// Function to process a WAV file
fn process_bro_file(file_path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}
pub struct Files {
    pub contents: Vec<Vec<u8>>,
    pub names: Vec<String>,
}


// Function to read and process files in a directory
pub fn dir_reader(directory_path: &Path) -> io::Result<Files> {
    let mut contents: Vec<Vec<u8>> = Vec::new();

    let mut names: Vec<String> = Vec::new();


    // Iterate through entries (files and subdirectories) in the given directory
    for entry in fs::read_dir(directory_path)? {
        // Unwrap the entry to get the actual entry information
        let entry = entry?;

        // Get the path of the entry (file or directory)
        let file_path = entry.path();

        // Add the filename to the list
        if let Some(filename) = file_path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                names.push(filename_str.to_string());
            }
        }

        // Check if the file is a WAV file
        contents.push(read_file(&file_path)?);
    }
    Ok(Files {contents, names})
}

pub fn read_file(file_path: &Path) -> Result<Vec<u8>, Error> {
    if is_bro_file(file_path)? {
        // If it's a WAV file, process it using the process_wav_file function
        Ok(process_bro_file(file_path)?)
    }
    else {
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
        let n = file.by_ref().take(chunk_size as u64).read_to_end(&mut chunk)?;
        if n == 0 { break; }
        list_of_chunks.push(chunk);
        if n < chunk_size { break; }
    }
    Ok(())
}