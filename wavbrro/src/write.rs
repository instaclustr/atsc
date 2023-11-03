use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::path::Path;

pub fn write_wavbrro_file(file_path: &Path, content: &[u8]) {
    // The content of the header
    let header: [u8; 12] = *b"WBRO0000WBRO";
    // We need to put the header in front
    let file = File::create(file_path).expect("Can't create file!");
    file.write_at(&header, 0).expect("Fail to write header");
    file.write_at(content, header.len() as u64).expect("Fail to write content");
}