// implement a streaming writer here

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

// Function to create a streaming writer for a file
fn create_streaming_writer(file_path: &Path) -> io::Result<File> {
    // Open the file for writing, creating it if it doesn't exist
    File::create(file_path)
}

// Function to write data to a streaming writer
fn write_data_to_stream(writer: &mut File, data: &[u8]) -> io::Result<()> {
    writer.write_all(data)?;
    Ok(())
}
fn main() -> io::Result<()> {
    // Define the file path where data will be written
    let file_path = Path::new(""); // Replace with the actual file path

    // Create a streaming writer for the file
    let mut writer = create_streaming_writer(&file_path)?;

    // Data to be written
    let data_to_write = b"";

    // Write data to the streaming writer
    write_data_to_stream(&mut writer, data_to_write)?;


    writer.flush()?; // Flush any buffered data
    // The writer will be automatically closed when it goes out of scope

    Ok(())
}
