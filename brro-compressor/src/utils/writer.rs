use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::TempDir; // Import the tempfile crate

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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stream_writer() {
        // Create a temporary directory to hold the test file
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test.txt");

        // Write data to the streaming writer
        let data_to_write = b"Hello, World!\n";
        {
            let mut writer = create_streaming_writer(&test_file_path).expect("Failed to create writer");
            write_data_to_stream(&mut writer, data_to_write).expect("Failed to write data");
            writer.flush().expect("Failed to flush data");
        }

        // Read the contents of the written file
        let file_contents = fs::read_to_string(&test_file_path).expect("Failed to read file");

        // Assert that the file contents match what was written
        assert_eq!(file_contents.as_bytes(), data_to_write);

        // Clean up the temporary directory and its contents
        temp_dir.close().expect("Failed to remove temporary directory");
    }
}
