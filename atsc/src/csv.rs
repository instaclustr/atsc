use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Csv(csv::Error),

    #[error("Timestamp filed is not found")]
    TimestampFieldNotFound,

    #[error("Value field is not found")]
    ValueFieldNotFound,

    #[error("Parsing timestamp is failed")]
    ParsingTimestampFailed,

    #[error("Parsing value is failed")]
    ParsingValueFailed,

    #[error("CSV record is missing field at index {index}")]
    MissingField { index: usize },

    #[error("CSV parser reported an inconsistent I/O error: {0}")]
    InconsistentIo(String),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub struct Sample {
    pub timestamp: i64,
    pub value: f64,
}

/// read_samples_with_headers reads samples from the given file.
/// It expects that timestamps are stored under timestamp_field field header
/// and values are stored under value_field.
pub fn read_samples_with_headers(
    filepath: &Path,
    timestamp_field: &str,
    value_field: &str,
) -> Result<Vec<Sample>> {
    let mut reader = open_csv_reader(filepath, true)?;
    let headers = reader.headers().map_err(classify_csv_error)?;

    // Find the index of the timestamp and value fields
    let _timestamp_idx = headers
        .iter()
        .position(|h| h == timestamp_field)
        .ok_or(Error::TimestampFieldNotFound)?;

    let value_idx = headers
        .iter()
        .position(|h| h == value_field)
        .ok_or(Error::ValueFieldNotFound)?;

    // Collect samples
    let mut samples = Vec::new();
    for record in reader.records() {
        let record = record.map_err(classify_csv_error)?;
        let value = parse_value(&record, value_idx)?;

        samples.push(Sample {
            timestamp: 0,
            value,
        });
    }

    Ok(samples)
}

/// read_samples reads samples from the given file.
/// It assumes that file contains no headers and
/// consists of only a single field with values.
pub fn read_samples(filepath: &Path) -> Result<Vec<Sample>> {
    let mut reader = open_csv_reader(filepath, false)?;

    // Collect samples
    let mut samples = Vec::new();
    for record in reader.records() {
        let record = record.map_err(classify_csv_error)?;
        let value = parse_value(&record, 0)?;

        samples.push(Sample {
            timestamp: 0,
            value,
        });
    }

    Ok(samples)
}

fn parse_value(record: &csv::StringRecord, index: usize) -> Result<f64> {
    record
        .get(index)
        .ok_or(Error::MissingField { index })?
        .parse()
        .map_err(|_| Error::ParsingValueFailed)
}

fn classify_csv_error(error: csv::Error) -> Error {
    if !error.is_io_error() {
        return Error::Csv(error);
    }

    let message = error.to_string();
    match error.into_kind() {
        csv::ErrorKind::Io(error) => Error::Io(error),
        _ => Error::InconsistentIo(message),
    }
}

fn open_csv_reader(filepath: &Path, has_headers: bool) -> Result<csv::Reader<File>> {
    let file = OpenOptions::new().read(true).open(filepath)?;

    let reader = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(file);
    Ok(reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    fn create_csv_file(content: &str, filepath: &Path) {
        let mut file = File::create(filepath).expect("Failed to create test CSV file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to test CSV file");
    }

    fn create_csv_bytes(content: &[u8], filepath: &Path) {
        let mut file = File::create(filepath).expect("Failed to create test CSV file");
        file.write_all(content)
            .expect("Failed to write to test CSV file");
    }

    #[test]
    fn test_valid_csv() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.into_path().join("test_valid.csv");

        let content = "timestamp,value\n1625097600,123.45\n1625184000,678.90\n";
        create_csv_file(content, &filepath);

        let result = read_samples_with_headers(&filepath, "timestamp", "value");
        assert!(result.is_ok());

        let samples = result.unwrap();
        assert_eq!(
            samples,
            vec![
                Sample {
                    timestamp: 0,
                    value: 123.45
                },
                Sample {
                    timestamp: 0,
                    value: 678.90
                },
            ]
        );
    }

    #[test]
    fn test_single_column_csv_no_headers() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.into_path().join("test_single_column.csv");

        let content = "123.45\n678.90\n";
        create_csv_file(content, &filepath);

        let result = read_samples(&filepath);
        assert!(result.is_ok());

        let samples = result.unwrap();
        assert_eq!(
            samples,
            vec![
                Sample {
                    timestamp: 0,
                    value: 123.45
                },
                Sample {
                    timestamp: 0,
                    value: 678.90
                },
            ]
        );
    }

    #[test]
    fn test_incorrect_format_single_column() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.into_path().join("test_incorrect_format.csv");

        let content = "value\n123.45\ninvalid_value\n678.90\n";
        create_csv_file(content, &filepath);

        let result = read_samples(&filepath);
        assert!(matches!(result, Err(Error::ParsingValueFailed)));
    }

    #[test]
    fn test_missing_timestamp_column() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir
            .into_path()
            .join("test_missing_timestamp_column.csv");

        let content = "time,value\n1625097600,123.45\n1625184000,678.90\n";
        create_csv_file(content, &filepath);

        let result = read_samples_with_headers(&filepath, "timestamp", "value");
        assert!(matches!(result, Err(Error::TimestampFieldNotFound)));
    }

    #[test]
    fn test_missing_value_column() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.into_path().join("test_missing_value_column.csv");

        let content = "timestamp,price\n1625097600,123.45\n1625184000,678.90\n";
        create_csv_file(content, &filepath);

        let result = read_samples_with_headers(&filepath, "timestamp", "value");
        assert!(matches!(result, Err(Error::ValueFieldNotFound)));
    }

    #[test]
    fn test_parsing_error_value() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.into_path().join("test_parsing_error_value.csv");

        let content = "timestamp,value\n1625097600,invalid_value\n1625184000,678.90\n";
        create_csv_file(content, &filepath);

        let result = read_samples_with_headers(&filepath, "timestamp", "value");
        assert!(matches!(result, Err(Error::ParsingValueFailed)));
    }

    #[test]
    fn test_unopenable_file() {
        let filepath = Path::new("/invalid/path/to/non_existent_file.csv");

        let result = read_samples_with_headers(filepath, "timestamp", "value");
        assert!(matches!(result, Err(Error::Io(_))));

        let result = read_samples(filepath);
        assert!(matches!(result, Err(Error::Io(_))));
    }

    #[test]
    fn test_no_headers_csv() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.into_path().join("test_no_headers.csv");

        let content = "timestamp,value\n1625097600,123.45\n1625184000,678.90\n";
        create_csv_file(content, &filepath);

        let result = read_samples(&filepath);
        assert!(matches!(result, Err(Error::ParsingValueFailed)));
    }

    #[test]
    fn malformed_utf8_returns_csv_error() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.path().join("malformed_utf8.csv");
        create_csv_bytes(b"timestamp,value\n1,\xff\n", &filepath);

        let result = read_samples_with_headers(&filepath, "timestamp", "value");

        assert!(matches!(
            result,
            Err(Error::Csv(error))
                if matches!(error.kind(), csv::ErrorKind::Utf8 { .. })
        ));
    }

    #[test]
    fn missing_record_field_preserves_unequal_lengths_error() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.path().join("missing_record_field.csv");
        create_csv_file("timestamp,value\n1\n", &filepath);

        let result = read_samples_with_headers(&filepath, "timestamp", "value");

        assert!(matches!(
            result,
            Err(Error::Csv(error))
                if matches!(error.kind(), csv::ErrorKind::UnequalLengths { .. })
        ));
    }

    #[test]
    fn missing_field_returns_typed_error() {
        let record = csv::StringRecord::new();

        assert!(matches!(
            parse_value(&record, 0),
            Err(Error::MissingField { index: 0 })
        ));
    }
}
