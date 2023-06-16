use std::{error::Error, fs::File};
use std::fs::{OpenOptions, metadata};
use chrono::{DateTime, Utc};
use hound::{WavWriter, WavSpec};

// --- Write layer
// Remote write spec: https://prometheus.io/docs/concepts/remote_write_spec/
pub struct WavMetric {
    metric_name: String,      // Metric name provided by prometheus
    instance: String,         // Instance name provided by prometheus
    job: String,              // Job name provided by prometheus 
    timeseries_data: Vec<(i64, f64)>, // Sample Data
    creation_time: String,    // The timestamp that this structure was created.
    last_file_created: Option<String> // Name of the last file created, !! might not make sense anymore !!
}
// Here is where things get tricky. Either we have a single strutcure and implement several WavWriters or we segment at the metric collection level.
// The advantage of implementing at the writing level is that we can look into the data and make a better guess based on the data.
// There is also the problem of not understanding the data clearly, or not having the WHOLE data available and making assumptions on
// a incomplete dataset. 
// Another way we can/should get around this would be "hinting" for the data type.
// If we are dealing with percentages we can go with i16, etc.
// Option B, less optimal, but more functional, is breaking f64 in 16bit parts and storing each part in its own channel. 
// We are choosing option B!

impl WavMetric {
    pub fn new(name: String, source: String, job: String) -> WavMetric {
        // Creation time
        let now: DateTime<Utc> = Utc::now();
        WavMetric { metric_name: name,
                    instance: source,
                    job,
                    timeseries_data: Vec::new(),
                    creation_time: now.format("%Y-%m-%d").to_string(),
                    last_file_created: None }
    }
    /// Flushes the metric to a WAV file
    /// TODO: Unwrap hell in here. Fix it later
    /// Too many assumptions on correct behavior of all the code. Assumption is the mother of all... Needs to be fixed
    pub fn flush(mut self) -> Result<(), ()> {
        if self.timeseries_data.is_empty() {
            // Can't flush empty data
            return Err(());
        }
        // Append if file exists, otherwise create spec and flush a new file
        let mut wav_writer = match self.last_file_created.is_none() {
            true => self.create_file().unwrap(),
            false => {    
                let file = OpenOptions::new().write(true).read(true).open(self.last_file_created.unwrap()).unwrap();
                WavWriter::new_append(file).unwrap() 
            }
            
        };
        // TODO: Check if the timestamp is one day ahead, if so, create another file
        // TODO: Deal with results too
        for (ts, sample ) in self.timeseries_data.drain(..) {
            let channel_data = WavMetric::split_f64_into_i16s(sample);
            // Write the samples interleaved
            for sample in channel_data {
                wav_writer.write_sample(sample);
            }
        }
        wav_writer.finalize();
        Ok(())
    }

    /// Create a file accordingly to the day of the year, the metric and the instance that generated the metric
    fn create_file(&mut self) -> Result<WavWriter<File>, hound::Error> {
        let spec = WavMetric::generate_wav_header(None);
        let file_name = format!("{}_{}_{}.wav", self.metric_name,self.instance, self.creation_time);
        let file_path = format!("./{}", file_name);
        // Create a new WAV file, if exists or open the existing one
        if let Ok(meta) = metadata(&file_path) {
            if meta.is_file() {
                let file = OpenOptions::new().write(true).open(&file_path)?;
                let wav_writer = WavWriter::new_append(file)?;
                return Ok(wav_writer);
            }
        }
        let file = OpenOptions::new().write(true).create(true).read(true).open(&file_path)?;
        let wav_writer = WavWriter::new(file, spec)?;
        self.last_file_created = Some(file_path);
        Ok(wav_writer)
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

    /// Add a single metric value to the structure
    pub fn add_timeseries(mut self, ts: i64, value: f64){
        self.timeseries_data.push((ts,value))
    }

    /// Add a vector of data to the existing timeseries
    pub fn add_bulk_timeseries(&mut self, timeseries: &mut Vec<(i64, f64)>){
        self.timeseries_data.append(timeseries)
    }

    /// Read a range in the structure
    pub fn get_range(self, ts_start: i64, ts_end: i64) -> Vec<(i64, f64)>{
        let mut i = 0;
        let mut j = 0;
        let mut count = 0;
        for (ts, _) in self.timeseries_data.iter() {
            if *ts < ts_start {i = count}
            if *ts < ts_end {j = count; break}
            count += 1;
        }
        if i > 0 { return self.timeseries_data[i-1..j].to_vec();}
        return self.timeseries_data[..j].to_vec();
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
}




