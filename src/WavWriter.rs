use std::error::Error;
use std::fs::OpenOptions;
use chrono::{DateTime, Utc};
use hound::{WavWriter, WavSpec};

// --- Write layer
// Remote write spec: https://prometheus.io/docs/concepts/remote_write_spec/
struct WavMetric {
    metric_name: String,      // Metric name provided by prometheus
    instance: String,         // Instance name provided by prometheus
    job: String,              // Job name provided by prometheus 
    timeseries_data: Option<Vec<WavData>>, // Sample Data
    append: bool,             // Internal structure, if TRUE a file exists, only needed to append to existing file
    creation_time: String,    // The timestamp that this structure was created.
    last_file_created: Option<String> // Name of the last file created
}
// Here is where things get tricky. Either we have a single strutcure and implement several WavWriters or we segment at the metric collection level.
// The advantage of implementing at the writing level is that we can look into the data and make a better guess based on the data.
// There is also the problem of not understanding the data clearly, or not having the WHOLE data available and making assumptions on
// a incomplete dataset. 
// Another way we can/should get around this would be "hinting" for the data type.
// If we are dealing with percentages we can go with i16, etc.
// Option B, less optimal, but more functional, is breaking f64 in 16bit parts and storing each part in its own channel.
//
struct WavData {
    timestamp: i64,
    value: f64
}

impl WavMetric {
    pub fn new(name: String, source: String, job: String) -> WavMetric {
        // Creation time
        let now: DateTime<Utc> = Utc::now();
        WavMetric { metric_name: name,
                    instance: source,
                    job,
                    timeseries_data: None,
                    append: false,
                    creation_time: now.format("%Y-%m-%d").to_string(),
                    last_file_created: None }
    }
    /// Flushes the metric to a WAV file
    pub fn flush(self) -> Result<(), ()> {
        if self.timeseries_data.is_none() {
            // Can't flush empty data
            return Err(());
        }
        // Append if file exists, otherwise create spec and flush a new file
        for sample in self.timeseries_data.unwrap() {
            let channel_data = split_f64_into_i16s(sample.value);
        }
        Ok(())
    }

    fn create_file() {

    }

    /// Generate the WAV file header.
    fn generate_wav_header(channels: Option<i32>) -> WavSpec {
        let spec = hound::WavSpec {
            channels: channels.unwrap_or(4) as u16,
            sample_rate: 8000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int
        };
        return spec;
}    
}



/// Instead of chasing data types and converting stuff, let's just unpack the f64 and 
/// put it into different channels. This way we can always garantee a clean i16 Wave file
fn split_f64_into_i16s(value: f64) -> [i16; 4] {
    let bits: u64 = value.to_bits();
    
    let i16_1 = (bits & 0xFFFF) as i16;
    let i16_2 = ((bits >> 16) & 0xFFFF) as i16;
    let i16_3 = ((bits >> 32) & 0xFFFF) as i16;
    let i16_4 = ((bits >> 48) & 0xFFFF) as i16;
    
    [i16_1, i16_2, i16_3, i16_4]
}

/// Recreate a f64
fn create_f64_from_16bits(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64) |
                   ((bits[1] as u64) << 16) |
                   ((bits[2] as u64) << 32) |
                   ((bits[3] as u64) << 48);
    
    let f64_value = f64::from_bits(u64_bits);
    
    f64_value
}
