use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

// Function to create a streaming writer for a file
pub fn create_streaming_writer(file_path: &Path) -> io::Result<File> {
    // Open the file for writing, creating it if it doesn't exist
    File::create(file_path)
}

// Function to write data to a streaming writer
pub fn write_data_to_stream(writer: &mut File, data: &[u8]) -> io::Result<()> {
    // Convert each byte to a string representation and collect them into a Vec<String>
    let strings: Vec<String> = data.iter().map(|&byte| byte.to_string()).collect();

    // Join the Vec<String> into a single String, separated by spaces (or any other delimiter you prefer)
    let output = strings.join(" ");

    writer.write_all(output.as_bytes())?;
    Ok(())
}
pub fn initialize_directory(base_dir: &Path) -> io::Result<()> {
    if !base_dir.exists() {
        fs::create_dir_all(base_dir)?;
    }
    Ok(())
}
pub fn replace_extension(filename: &String, new_extension: &str) -> String {
    let path = Path::new(&filename);
    let without_extension = path.file_stem().unwrap_or_default(); // gets the filename without extension
    let mut new_path = PathBuf::from(without_extension);
    new_path.set_extension(new_extension);
    new_path.to_string_lossy().into_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_stream_writer() {
        // Create a temporary directory to hold the test file
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test.txt");

        // Write data to the streaming writer
        let data_to_write = b"Hello, World!\n";
        {
            let mut writer =
                create_streaming_writer(&test_file_path).expect("Failed to create writer");
            write_data_to_stream(&mut writer, data_to_write).expect("Failed to write data");
            writer.flush().expect("Failed to flush data");
        }

        // Read the contents of the written file
        let file_contents = std::fs::read_to_string(&test_file_path).expect("Failed to read file");

        // Assert that the file contents match what was written
        assert_eq!(file_contents.as_bytes(), data_to_write);

        // Clean up the temporary directory and its contents
        temp_dir
            .close()
            .expect("Failed to remove temporary directory");
    }
}
