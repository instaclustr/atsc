use core::time;
/// All the utils/code related the to file management
/// 
/// ASSUMPTION: EACH DAY HAS 1 FILE!!! If this assumption change, change this file!
/// TODO: (BIG ONE!) Make this time period agnostic (so it would work with days, weeks, etc)
/// For a READ request that needs data for MetricX from Ta to Tb this would do the following:
/// 1. Do we have metricX? -> No, stop.
/// 2. Which file has Ta, and which has Tb?
///     2.1 Select them to read
/// 3. Read the indexes, and retrieve the available samples
/// 
/// Suggested internal Data Structure of the WAV file
/// 
/// +--------------------------------------------------------------------------------------------------------+
/// | HEADER | i16 @Chan1 | i16 @Chan2 | i16 @Chan3 | i16 @Chan4 | tick @Chan5 | i16 @Chan1 | i16 @Chan2 |...|
/// +--------------------------------------------------------------------------------------------------------+
///                 
///  Prometheus Point: f64 split into 4x i16 (channel 1 to 4) Timestamp: Tick into Channel 5
/// 

use std::fs::{self, File};
use std::mem;
use chrono::{DateTime, Utc, Duration, Datelike};

use crate::flac_reader::FlacMetric;
use crate::lib_vsri::{VSRI, day_elapsed_seconds, start_day_ts};

struct DateRange(DateTime<Utc>, DateTime<Utc>);

// Iterator for Day to Day
// TODO: move this to several impl? So we can return iterators over several time periods?
impl Iterator for DateRange {
    type Item = DateTime<Utc>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 <= self.1 {
            let next = self.0 + Duration::days(1);
            Some(mem::replace(&mut self.0, next))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PromDataPoint {
    pub point: f64,
    pub time: i64,
}

impl PromDataPoint {
    pub fn new(data: f64, timestamp: i64) -> Self {
        PromDataPoint {
            point: data,
            time: timestamp
        }
    }

    /// This will return a data point from a FLAC file for the provided point in time
    pub fn read_data_point(file: &File) -> PromDataPoint {
        let data_point = PromDataPoint {
            point: 0.0,
            time: 0,
        };
        data_point
    }
}


/// Given a metric name and a time interval, returns all the files handles for the files that contain that data
pub fn get_file_names(metric_name: &str, start_time: i64, end_time: i64) -> Option<Vec<(File, VSRI)>> {
    let mut file_index_vec = Vec::new();
    let start_date = DateTime::<Utc>::from_utc(
                                            chrono::NaiveDateTime::from_timestamp_opt((start_time/1000).into(), 0).unwrap(),
                                              Utc,
                                                    );
    let end_date = DateTime::<Utc>::from_utc(
                                          chrono::NaiveDateTime::from_timestamp_opt((end_time/1000).into(), 0).unwrap(),
                                            Utc,
                                                    );
    for date in DateRange(start_date, end_date) {
        let day = date.day();
        let month = date.month();
        let year = date.year();
        let data_file_name = format!("{}_{}_{}_{}",metric_name, day, month, year);
        let vsri = VSRI::load(&data_file_name);
        let file = match  fs::File::open(format!("{}.flac", data_file_name.clone())) {
            Ok(file) => {
                file
            },
            Err(_err) => {
                println!("File {} doesn't exist, skipping", data_file_name); 
                continue; 
            }
         };
         // If I got here, I should be able to unwrap VSRI safely.
         file_index_vec.push((file, vsri.unwrap()));
    }
    // We have at least one file
    if file_index_vec.len() >= 1 {
        return Some(file_index_vec);
    }
    None
}

/// Retrieves all the available data points in a timerange in the provided Vector of files and indexes
pub fn get_data_between_timestamps(start_time: i64, end_time: i64, file_vec: Vec<(File, VSRI)>) -> Vec<PromDataPoint> {
    let mut data_points = Vec::new();
    /* Processing logic:
        Case 1 (2+ files):
         The first file, the period if from `start_time` to end of the file (use index),
         The second until the last file (exclusive), we need all the data points we can get (read full file).
         The last file we need from start until the `end_time` (use index).
        Case 2 (Single file):
         Read the index to locate the start sample and the end sample.
         Read the file and obtain said samples.
     */
    // How many files to process
    let file_count = file_vec.len();
    // Get the baseline timestamps to add to the index timestamps
    let start_date = DateTime::<Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp_opt((start_time/1000).into(), 0).unwrap(),
          Utc,
                );
    let end_date = DateTime::<Utc>::from_utc(
    chrono::NaiveDateTime::from_timestamp_opt((end_time/1000).into(), 0).unwrap(),
        Utc,
                );
    let ts_bases: Vec<i64> = DateRange(start_date, end_date).map(|dt| start_day_ts(dt)).collect();
    let start_ts_i32 = day_elapsed_seconds(start_time);
    let end_ts_i32 = day_elapsed_seconds(end_time);
    // Where the samples land in the indexes
    let mut samples_locations = [0, 0];
    for pack in file_vec.into_iter().enumerate() {
        let iter_index = pack.0;
        let file = pack.1.0;
        let vsri = pack.1.1;
        if file_count == 1 {
            // Case 2
            // get_sample can return None
            let start_sample = vsri.get_this_or_next(start_ts_i32);
            if start_sample.is_none() {
                // No sample in the file fits the current requested interval
                return data_points;
            }
            // If I can start reading the file, I can get at least one sample, so it is safe to unwrap.
            let end_sample = vsri.get_this_or_previous(end_ts_i32).unwrap();
            samples_locations = [start_sample.unwrap(), end_sample];
        } else {
        // Case 1
            match pack.0 {
                // First file
                0 => {
                    let start_sample = vsri.get_this_or_next(start_ts_i32);
                    if start_sample.is_none() { continue; }
                    samples_locations = [start_sample.unwrap(), vsri.get_sample_count()];
                },
                // Last file
                _ if iter_index == file_count-1 => {
                    let end_sample = vsri.get_this_or_previous(end_ts_i32);
                    if end_sample.is_none() { continue; }
                    samples_locations = [0, end_sample.unwrap()];
                },
                // Other files
                _ => {
                    // Collect the full file
                    samples_locations = [0, vsri.get_sample_count()];
                }
            }
        }
        // Collect the data points
        let flac_metric = FlacMetric::new(file, start_time);
        let tmp_vec = vsri.get_all_timestamps();
        let start = samples_locations[0];
        let end = samples_locations[1];
        // !@)(#*&!@)# usize and ints...
        let time_for_samples = &tmp_vec[start as usize..=end as usize];
        // The time I learned if..else is an expression!
        let temp_result = if start == 0 && end == vsri.get_sample_count() {
            flac_metric.get_all_samples()
        } else {
            flac_metric.get_samples(Some(start), Some(end))
        };

        match temp_result {
            // Pack this into DataPoints
            Ok(samples) => {
                for (v, t) in samples.into_iter().zip(time_for_samples.into_iter()) {
                    let ts = *t as i64+ts_bases[iter_index];
                    data_points.push(PromDataPoint::new(v, ts));
                }
            },
            Err(err) => {println!("[DEBUG][READ] Error processing FLaC file {:?}", err); continue;}
        }
    }
    data_points
}