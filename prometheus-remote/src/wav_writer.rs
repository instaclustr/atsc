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

use chrono::{DateTime, Utc};
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::fs::{metadata, OpenOptions};
use std::process::Command;

use crate::lib_vsri::{day_elapsed_seconds, Vsri};

// --- Write layer
// Remote write spec: https://prometheus.io/docs/concepts/remote_write_spec/
pub struct WavMetric {
    pub metric_name: String,               // Metric name provided by prometheus
    pub instance: String,                  // Instance name provided by prometheus
    pub job: String,                       // Job name provided by prometheus
    pub timeseries_data: Vec<(i64, f64)>,  // Sample Data
    pub creation_time: String,             // The timestamp that this structure was created.
    pub last_file_created: Option<String>, // Name of the last file created, !! might not make sense anymore !!
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
    /// Create a new WavMetric struct. `start_sample_ts` *must be* timestamp with miliseconds!
    pub fn new(name: String, source: String, job: String, start_sample_ts: i64) -> WavMetric {
        // Sample needs to fall within the file that the TS refers too, not the calendar day
        let start_date = DateTime::<Utc>::from_timestamp(start_sample_ts / 1000, 0).unwrap();
        // TODO: Do not ignore JOB!
        WavMetric {
            metric_name: name,
            instance: source,
            job,
            timeseries_data: Vec::new(),
            creation_time: start_date.format("%Y-%m-%d").to_string(),
            last_file_created: None,
        }
    }
    /// Flushes the metric to a WAV file
    // TODO: Unwrap hell in here. Need better error control
    // Too many assumptions on correct behavior of all the code. Assumption is the mother of all... Needs to be fixed
    pub fn flush(mut self) -> Result<(), i32> {
        let mut processed_samples: i32 = 0;
        let vsri: Option<Vsri>;
        if self.timeseries_data.is_empty() {
            // Can't flush empty data
            error!("[WRITE][WAV] Call flush on empty data");
            return Err(processed_samples);
        }
        // Append if file exists, otherwise create spec and flush a new file
        let mut wav_writer = match self.last_file_created.is_none() {
            true => {
                let handlers = self.create_file().unwrap();
                vsri = Some(handlers.1);
                handlers.0
            }
            false => {
                let file = OpenOptions::new()
                    .write(true)
                    .read(true)
                    .open(self.last_file_created.unwrap())
                    .unwrap();
                // Load the index file
                // TODO: one more unwrap to work on later
                vsri = Some(Vsri::load(&self.metric_name).unwrap());
                WavWriter::new_append(file).unwrap()
            }
        };
        // TODO: #12 Check if the timestamp is one day ahead, if so, create another file, and pack the previous one as FLAC
        let vsri_unwrapped = &mut vsri.unwrap();
        let mut int_r: Result<(), i32> = Ok(());
        for (ts, sample) in self.timeseries_data {
            let short_ts = ts / 1000;
            let r = vsri_unwrapped.update_for_point(day_elapsed_seconds(short_ts));
            if r.is_err() {
                // Period changed (default: day)
                warn!("[WRITE][WAV] Detected a day change while processing samples. Wrote {} before error.", processed_samples);
                int_r = Err(processed_samples);
                break;
            }
            let channel_data = WavMetric::split_f64_into_i16s(sample);
            // Write the samples interleaved
            for sample in channel_data {
                let ww = wav_writer.write_sample(sample);
                if ww.is_err() {
                    error!(
                        "[WAVWRITER] Unable to write sample {:?} in file {:?}!",
                        sample, self.metric_name
                    );
                    return Err(processed_samples);
                }
            }
            processed_samples += 1;
        }
        debug!("[WRITE][WAV] Wrote {} samples", processed_samples);
        // TODO: Process there errors too, create different errors here
        let r = vsri_unwrapped.flush();
        if r.is_err() {
            error!(
                "[WAVWRITER] Unable to flush VSRI for {:?}!",
                self.metric_name
            );
            panic!(
                "[WAVWRITER] Failed flushing index. Lost information. {}",
                r.unwrap_err()
            )
        }
        let r2 = wav_writer.finalize();
        if r2.is_err() {
            error!(
                "[WAVWRITER] Unable to flush WAV file {:?}!",
                self.metric_name
            );
            panic!(
                "[WAVWRITER] Failed flushing file. Lost information. {}",
                r.unwrap_err()
            )
        }
        int_r
    }

    /// Create a file accordingly to the day of the year, the metric and the instance that generated the metric
    /// TODO: Create file shouldn't open a file for append. Should only create. Fix this (or rename)
    fn create_file(&mut self) -> Result<(WavWriter<File>, Vsri), hound::Error> {
        let spec = WavMetric::generate_wav_header(None);
        let file_name = format!(
            "{}_{}_{}",
            self.metric_name, self.instance, self.creation_time
        );
        let file_path = format!("./{}.wav", file_name);
        // Create a new WAV file, if exists or open the existing one
        if let Ok(meta) = metadata(&file_path) {
            if meta.is_file() {
                let file = OpenOptions::new().write(true).read(true).open(&file_path)?;
                let wav_writer = WavWriter::new_append(file)?;
                return Ok((wav_writer, Vsri::load(&file_name).unwrap()));
            }
        }
        let file = File::create(&file_path)?;
        let wav_writer = WavWriter::new(file, spec)?;
        self.last_file_created = Some(file_path);
        // TODO: Y can't be 0. Needs to be TS
        Ok((wav_writer, Vsri::new(&file_name)))
    }

    /// Generate the WAV file header.
    fn generate_wav_header(channels: Option<i32>) -> WavSpec {
        hound::WavSpec {
            channels: channels.unwrap_or(4) as u16,
            sample_rate: 8000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        }
    }

    /// Add a single metric value to the structure
    pub fn add_timeseries(mut self, ts: i64, value: f64) {
        self.timeseries_data.push((ts, value))
    }

    /// Add a vector of data to the existing timeseries
    pub fn add_bulk_timeseries(&mut self, timeseries: &mut Vec<(i64, f64)>) {
        self.timeseries_data.append(timeseries)
    }

    /// Read a range in the structure
    pub fn get_range(self, ts_start: i64, ts_end: i64) -> Vec<(i64, f64)> {
        let mut i = 0;
        let mut j = 0;
        for (count, (ts, _)) in self.timeseries_data.iter().enumerate() {
            if *ts < ts_start {
                i = count
            }
            if *ts < ts_end {
                j = count;
                break;
            }
        }
        if i > 0 {
            return self.timeseries_data[i - 1..j].to_vec();
        }
        self.timeseries_data[..j].to_vec()
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
        let u64_bits = (bits[0] as u64)
            | ((bits[1] as u64) << 16)
            | ((bits[2] as u64) << 32)
            | ((bits[3] as u64) << 48);

        f64::from_bits(u64_bits)
    }

    /// Rotate the wav file after the interval and save it as a FLaC file
    fn rotate_wav_into_flac(self) {
        let file_in = format!(
            "{}_{}_{}.wav",
            self.metric_name, self.instance, self.creation_time
        );
        let file_out = format!(
            "{}_{}_{}.flac",
            self.metric_name, self.instance, self.creation_time
        );
        // Command: sox input.wav output.flac
        let output = Command::new("sox")
            .arg(file_in)
            .arg(file_out)
            .output()
            .expect("Error converting WAV to FLAC");
        if !output.status.success() {
            panic!("Could not rotate file!")
        }
    }

    /// Check if the current timestamp is within the file period
    fn is_ts_valid(_ts: i64) -> bool {
        true
    }
}
