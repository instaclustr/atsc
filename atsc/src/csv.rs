use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::Path;

const DEFAULT_MAX_INPUT_BYTES: usize = 256 * 1024 * 1024;
const DEFAULT_MAX_SAMPLES: usize = 131_072 * u8::MAX as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvReadLimits {
    pub max_input_bytes: usize,
    pub max_samples: usize,
}

impl Default for CsvReadLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            max_samples: DEFAULT_MAX_SAMPLES,
        }
    }
}

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

    #[error("CSV input is {actual} bytes, above configured limit {limit}")]
    InputLimitExceeded { actual: usize, limit: usize },

    #[error("CSV contains {actual} samples, above configured limit {limit}")]
    SampleLimitExceeded { actual: usize, limit: usize },
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
    Ok(
        read_values_with_headers(filepath, timestamp_field, value_field)?
            .into_iter()
            .map(|value| Sample {
                timestamp: 0,
                value,
            })
            .collect(),
    )
}

pub fn read_values_with_headers(
    filepath: &Path,
    timestamp_field: &str,
    value_field: &str,
) -> Result<Vec<f64>> {
    read_values_with_headers_and_limits(
        filepath,
        timestamp_field,
        value_field,
        CsvReadLimits::default(),
    )
}

pub fn read_values_with_headers_and_limits(
    filepath: &Path,
    timestamp_field: &str,
    value_field: &str,
    limits: CsvReadLimits,
) -> Result<Vec<f64>> {
    let mut reader = open_csv_reader(filepath, true, limits)?;
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

    let mut values = Vec::new();
    for record in reader.records() {
        let record = record.map_err(classify_csv_error)?;
        let value = parse_value(&record, value_idx)?;
        push_value(&mut values, value, limits.max_samples)?;
    }

    Ok(values)
}

/// read_samples reads samples from the given file.
/// It assumes that file contains no headers and
/// consists of only a single field with values.
pub fn read_samples(filepath: &Path) -> Result<Vec<Sample>> {
    Ok(read_values(filepath)?
        .into_iter()
        .map(|value| Sample {
            timestamp: 0,
            value,
        })
        .collect())
}

pub fn read_values(filepath: &Path) -> Result<Vec<f64>> {
    read_values_with_limits(filepath, CsvReadLimits::default())
}

pub fn read_values_with_limits(filepath: &Path, limits: CsvReadLimits) -> Result<Vec<f64>> {
    let mut reader = open_csv_reader(filepath, false, limits)?;
    let mut values = Vec::new();
    for record in reader.records() {
        let record = record.map_err(classify_csv_error)?;
        let value = parse_value(&record, 0)?;
        push_value(&mut values, value, limits.max_samples)?;
    }

    Ok(values)
}

fn push_value(values: &mut Vec<f64>, value: f64, limit: usize) -> Result<()> {
    if values.len() == limit {
        return Err(Error::SampleLimitExceeded {
            actual: limit.saturating_add(1),
            limit,
        });
    }
    values.push(value);
    Ok(())
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
        csv::ErrorKind::Io(error) => {
            if let Some(limit) = error
                .get_ref()
                .and_then(|source| source.downcast_ref::<InputLimitIoError>())
            {
                return Error::InputLimitExceeded {
                    actual: limit.actual,
                    limit: limit.limit,
                };
            }
            Error::Io(error)
        }
        _ => Error::InconsistentIo(message),
    }
}

fn open_csv_reader(
    filepath: &Path,
    has_headers: bool,
    limits: CsvReadLimits,
) -> Result<csv::Reader<BoundedReader<File>>> {
    let file = OpenOptions::new().read(true).open(filepath)?;
    let actual = usize::try_from(file.metadata()?.len()).unwrap_or(usize::MAX);
    if actual > limits.max_input_bytes {
        return Err(Error::InputLimitExceeded {
            actual,
            limit: limits.max_input_bytes,
        });
    }

    let reader = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_reader(BoundedReader::new(file, limits.max_input_bytes));
    Ok(reader)
}

#[derive(Debug)]
struct InputLimitIoError {
    actual: usize,
    limit: usize,
}

impl std::fmt::Display for InputLimitIoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "input is {} bytes, above configured limit {}",
            self.actual, self.limit
        )
    }
}

impl std::error::Error for InputLimitIoError {}

struct BoundedReader<R> {
    inner: R,
    remaining: usize,
    limit: usize,
}

impl<R> BoundedReader<R> {
    fn new(inner: R, limit: usize) -> Self {
        Self {
            inner,
            remaining: limit,
            limit,
        }
    }
}

impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let read_limit = self.remaining.saturating_add(1).min(buffer.len());
        let read = self.inner.read(&mut buffer[..read_limit])?;
        if read > self.remaining {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                InputLimitIoError {
                    actual: self.limit.saturating_add(1),
                    limit: self.limit,
                },
            ));
        }
        self.remaining -= read;
        Ok(read)
    }
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

    #[test]
    fn value_only_reader_enforces_n_plus_one_sample_limit() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.path().join("sample-limit.csv");
        create_csv_file("1\n2\n3\n", &filepath);

        assert!(matches!(
            read_values_with_limits(
                &filepath,
                CsvReadLimits {
                    max_input_bytes: 6,
                    max_samples: 2,
                },
            ),
            Err(Error::SampleLimitExceeded {
                actual: 3,
                limit: 2
            })
        ));
    }

    #[test]
    fn value_only_reader_accepts_exact_byte_and_sample_limits() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.path().join("exact-limits.csv");
        let content = "1\n2\n";
        create_csv_file(content, &filepath);

        assert_eq!(
            read_values_with_limits(
                &filepath,
                CsvReadLimits {
                    max_input_bytes: content.len(),
                    max_samples: 2,
                },
            )
            .unwrap(),
            [1.0, 2.0]
        );
    }

    #[test]
    fn oversized_single_record_is_rejected_before_csv_record_allocation() {
        let temp_dir = tempdir().unwrap();
        let filepath = temp_dir.path().join("oversized-record.csv");
        create_csv_file("1234567890", &filepath);

        assert!(matches!(
            read_values_with_limits(
                &filepath,
                CsvReadLimits {
                    max_input_bytes: 9,
                    max_samples: 1,
                },
            ),
            Err(Error::InputLimitExceeded {
                actual: 10,
                limit: 9
            })
        ));
    }

    #[test]
    fn limited_value_reader_preserves_structural_and_utf8_errors() {
        let temp_dir = tempdir().unwrap();
        let structural = temp_dir.path().join("structural.csv");
        create_csv_file("timestamp,value\n1\n", &structural);
        assert!(matches!(
            read_values_with_headers_and_limits(
                &structural,
                "timestamp",
                "value",
                CsvReadLimits::default(),
            ),
            Err(Error::Csv(error))
                if matches!(error.kind(), csv::ErrorKind::UnequalLengths { .. })
        ));

        let utf8 = temp_dir.path().join("utf8.csv");
        create_csv_bytes(b"timestamp,value\n1,\xff\n", &utf8);
        assert!(matches!(
            read_values_with_headers_and_limits(
                &utf8,
                "timestamp",
                "value",
                CsvReadLimits::default(),
            ),
            Err(Error::Csv(error))
                if matches!(error.kind(), csv::ErrorKind::Utf8 { .. })
        ));
    }
}
