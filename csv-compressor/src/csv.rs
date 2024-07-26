use csv::Reader;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::error::Error;
use std::io;
use std::rc::Rc;

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

/// SampleParser is responsible for reading and parsing CSV formatted samples from R
pub struct SampleParser<R: io::Read> {
    reader: R,
}

impl<R: io::Read> SampleParser<R> {
    pub fn new(reader: R) -> Self {
        SampleParser { reader }
    }

    /// Parses CSV formatted samples from src
    pub fn parse(&mut self) -> Result<Vec<Sample>, Box<dyn Error>> {
        let mut reader = Reader::from_reader(&mut self.reader);
        let mut samples = Vec::new();
        for result in reader.deserialize() {
            let sample: Sample = result?;
            samples.push(sample);
        }
        Ok(samples)
    }
}

/// SampleWriter is responsible for writing Sample as CSV into W
pub struct SampleWriter<W: io::Write> {
    writer: Rc<RefCell<W>>,
}

impl<W: io::Write> SampleWriter<W> {
    pub fn new(writer: Rc<RefCell<W>>) -> Self {
        SampleWriter { writer }
    }

    /// Writes samples into writer as CSV
    pub fn write_samples(&self, samples: &Vec<Sample>) -> Result<(), Box<dyn Error>> {
        let mut w = self.writer.borrow_mut();
        let mut writer = csv::Writer::from_writer(&mut *w);
        for sample in samples {
            writer.serialize(sample)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_sample_parser_parse() {
        let data = "timestamp,value\n1,1.01\n5,1.22\n";
        let cursor = Cursor::new(data);

        let mut parser = SampleParser::new(cursor);
        let samples = parser.parse().unwrap();

        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0], Sample::new(1, 1.01));
        assert_eq!(samples[1], Sample::new(5, 1.22));
    }

    #[test]
    fn test_sample_parser_parse_empty() {
        let data = "timestamp,value\n";
        let cursor = Cursor::new(data);

        let mut parser = SampleParser::new(cursor);
        let samples = parser.parse().unwrap();

        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_sample_parser_parse_invalid_data() {
        let data = "timestamp,value\n1,not_a_float\n";
        let cursor = Cursor::new(data);

        let mut parser = SampleParser::new(cursor);
        let result = parser.parse();

        assert!(result.is_err());
    }

    #[test]
    fn test_sample_writer() {
        let cursor = Rc::new(RefCell::new(Cursor::new(vec![])));

        let writer = SampleWriter::new(Rc::clone(&cursor));

        let samples = vec![Sample::new(1, 1.01), Sample::new(5, 1.31)];

        let res = writer.write_samples(&samples);
        assert!(res.is_ok());

        let expected = "timestamp,value\n1,1.01\n5,1.31\n";
        let result = String::from_utf8(cursor.borrow().get_ref().to_vec());

        assert!(res.is_ok());
        assert_eq!(expected, result.unwrap(), "result mismatch")
    }
}
