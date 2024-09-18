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

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::path::Path;

/// Sample describe a single metric record according to csv format.
/// The file should have the following structure:
/// | timestamp | value |
/// |    000001 | 1.01  |
/// |    000005 | 1.22  |
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub timestamp: i64,
    pub value: f64,
}

impl Sample {
    pub fn new(timestamp: i64, value: f64) -> Self {
        Sample { timestamp, value }
    }
}

/// Reads samples from csv file at dest.
/// The file should contain the following structure:
/// | timestamp | value |
/// |    000001 | 1.01  |
/// |    000005 | 1.22  |
pub fn read_samples_from_csv_file(dest: &Path) -> Result<Vec<Sample>, csv::Error> {
    let file = OpenOptions::new().read(true).open(dest)?;
    let mut reader = csv::Reader::from_reader(file);
    reader.deserialize().collect()
}

/// Writes samples to file at dest as csv
pub fn write_samples_to_csv_file(dest: &Path, samples: &[Sample]) -> Result<(), csv::Error> {
    let mut csv_file = File::create(dest)?;
    let mut writer = csv::Writer::from_writer(&mut csv_file);
    for sample in samples {
        writer.serialize(sample)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn test_write_samples_to_csv_file() {
        let expected_contents = "timestamp,value\n1,1.01\n5,1.22\n";
        let samples = vec![Sample::new(1, 1.01), Sample::new(5, 1.22)];

        let temp_dir =
            TempDir::new("test_write_samples").expect("Unable to create temporary directory");
        let path = temp_dir.path().join("samples.csv");

        let result = write_samples_to_csv_file(&path, &samples);
        assert!(result.is_ok());

        let contents = std::fs::read_to_string(&path).expect("Unable to read the file");
        assert_eq!(contents, expected_contents);
    }

    #[test]
    fn test_read_samples_from_csv_file() {
        let csv_content = "timestamp,value\n1,1.01\n5,1.22\n";
        let expected_samples = vec![Sample::new(1, 1.01), Sample::new(5, 1.22)];

        let temp_dir =
            TempDir::new("test_read_samples").expect("Unable to create temporary directory");
        let path = temp_dir.path().join("samples.csv");

        // Writing content to test file
        let mut file = File::create(&path).expect("Unable to create test file");
        file.write_all(csv_content.as_bytes())
            .expect("Unable to write data");

        let result = read_samples_from_csv_file(&path);
        assert!(result.is_ok());

        let samples = result.unwrap();
        assert_eq!(samples, expected_samples);
    }

    #[test]
    fn test_write_and_read_samples() {
        let samples = vec![Sample::new(1, 1.01), Sample::new(5, 1.22)];

        let temp_dir = TempDir::new("test_write_and_read_samples")
            .expect("Unable to create temporary directory");
        let path = temp_dir.path().join("samples.csv");

        let write_result = write_samples_to_csv_file(&path, &samples);
        assert!(write_result.is_ok());

        let read_result = read_samples_from_csv_file(&path);
        assert!(read_result.is_ok());

        let read_samples = read_result.unwrap();
        assert_eq!(samples, read_samples);
    }
}
