/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

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
