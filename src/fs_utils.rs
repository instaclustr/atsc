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
use chrono::{DateTime, Utc, Duration};

use crate::flac_reader::SimpleFlacReader;
use crate::lib_vsri::{VSRI, day_elapsed_seconds, start_day_ts, MAX_INDEX_SAMPLES};

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

#[derive(Debug, Clone, Copy)]
pub struct PromDataPoint {
    pub point: f64,
    pub time: i64,
}

impl PromDataPoint {
    /// Creates a new Prometheus Data Point. It assumes a timestamp with seconds since EPOCH, and converts internally to
    /// miliseconds since EPOCH.
    pub fn new(data: f64, timestamp: i64) -> Self {
        PromDataPoint {
            point: data,
            time: timestamp*1000
        }
    }
}
/// Holds a time range for the file and index
#[derive(Debug, Clone, Copy)]
struct FileTimeRange {
    start: i32,
    end: i32
}

impl FileTimeRange {
    fn new(start: i32, end: i32) -> Self {
        FileTimeRange {
            start,
            end
        }
    }
}

/// A struct that allows the precise location of data inside the file is in it
/// TODO: Make FILE a FlacReader
#[derive(Debug)]
pub struct DataLocator {
    file: File,
    index: VSRI,
    time_range: FileTimeRange,
    date: DateTime<Utc>
}

impl DataLocator {
    /// Creates a new DataLocator, includes the File, Index and the Time Range for the data it is expected to return.
    /// This is a lazy, doesn't check for the intersection between the time range and data owned until the data is 
    /// requested.
    fn new(file: File, index: VSRI, time_range: FileTimeRange, date: DateTime<Utc> ) -> Self {
        DataLocator {
            file,
            index,
            time_range,
            date
        }
    }

    /// Checks if the Locator time_range intersects with the Index data
    fn do_intersect(&self) -> bool {
        // If the data start after the end of the range or the data ends before the beggining of the range
        if self.index.min() > self.time_range.end || 
            self.index.max() < self.time_range.start { 
            return false;
        }
        // index function checks for no ownership, this function checks for ownership, invert the result
        !self.index.is_empty([self.time_range.start, self.time_range.end])
    }

    fn get_samples_from_range(&self) -> Option<[i32; 2]>{
        // By default, get all the samples
        let mut sample_range: [i32;2] = [0, MAX_INDEX_SAMPLES];
        if !self.do_intersect() {
            return None;
        }
        match self.time_range.start {
            0 => { sample_range[0] = 0;},
            _ => {
                // There is intersection, it can unwrap safely
                sample_range[0] = self.index.get_this_or_next(self.time_range.start).unwrap();
            },
        }
        match self.time_range.end {
            // Match cannot shadow statics and whatever
            _ if self.time_range.end == MAX_INDEX_SAMPLES => { sample_range[1] = self.index.get_sample_count();},
            _ => {
                // There is intersection, it can unwrap safely
                sample_range[1] = self.index.get_this_or_previous(self.time_range.start).unwrap();
            },
        }
        Some(sample_range)
    }

    /// Consumes the DataLocator to return a Vec of PromDataPoints
    pub fn into_prom_data_point(self) -> Vec<PromDataPoint>{
        let mut prom_data = Vec::new();
        let samples_locations = self.get_samples_from_range();
        let flac_metric = SimpleFlacReader::new(self.file, self.time_range.start as i64);
        let tmp_vec = self.index.get_all_timestamps();
        // There goes an empty arry
        if samples_locations.is_none() { return prom_data; }
        let start = samples_locations.unwrap()[0];
        let end = samples_locations.unwrap()[1]-1;
        debug!("[READ] Samples located! From {} to {}. TS available: {}",start, end, tmp_vec.len());
        let time_for_samples = &tmp_vec[start as usize..=end as usize];
        // The time I learned if..else is an expression!
        let temp_result = if start == 0 && end == self.index.get_sample_count() {
            flac_metric.get_all_samples()
        } else {
            flac_metric.get_samples(Some(start), Some(end))
        };
        match temp_result {
            // Pack this into DataPoints
            Ok(samples) => {
                for (v, t) in samples.into_iter().zip(time_for_samples.into_iter()) {
                    let ts = *t as i64+start_day_ts(self.date);
                    prom_data.push(PromDataPoint::new(v, ts*1000));
                }
            },
            Err(err) => {error!("[READ] Error processing FLaC file {:?}", err); return prom_data;}
        }
        prom_data
    }

    /// Given a metric name and a time interval, returns all the files handles for the files that *might* contain that data (No data range intersection is done here)
    pub fn get_locators_for_range(metric_name: &str, start_time: i64, end_time: i64) -> Option<Vec<DataLocator>> {
        let mut file_index_vec = Vec::new();
        let data_locator_vec: Vec<DataLocator>;
        let start_date = DateTime::<Utc>::from_utc(
                                                chrono::NaiveDateTime::from_timestamp_opt((start_time/1000).into(), 0).unwrap(),
                                                Utc,
                                                        );
        let end_date = DateTime::<Utc>::from_utc(
                                            chrono::NaiveDateTime::from_timestamp_opt((end_time/1000).into(), 0).unwrap(),
                                                Utc,
                                                        );
        let file_time_intervals = time_intervals(start_time, end_time);
        debug!("[READ] Time intervals for the range {:?} ", file_time_intervals);
        let mut range_count = 0;
        for date in DateRange(start_date, end_date).enumerate() {
            let data_file_name = format!("{}_{}",metric_name, date.1.format("%Y-%m-%d").to_string());
            debug!("[READ] Time intervals for file {}: {:?} ", data_file_name, file_time_intervals[range_count]);
            let vsri = VSRI::load(&data_file_name);
            range_count += 1;
            let file = match  fs::File::open(format!("{}.flac", data_file_name.clone())) {
                Ok(file) => {
                    file
                },
                Err(err) => {
                    warn!("[READ] Error processing {}.flac. Error: {}. Skipping file.", data_file_name, err); 
                    continue; 
                }
            };
            // If I got here, I should be able to unwrap VSRI safely.
            file_index_vec.push((file, vsri.unwrap(), date));
        }
        // Creating the Time Range array
        let start_ts_i32 = day_elapsed_seconds(start_time);
        let end_ts_i32 = day_elapsed_seconds(end_time);
        let mut time_intervals = Vec::new();
        match range_count {
            1 => { time_intervals.push(FileTimeRange::new(start_ts_i32, end_ts_i32));},
            2 => {
                time_intervals.push(FileTimeRange::new(start_ts_i32, MAX_INDEX_SAMPLES));
                time_intervals.push(FileTimeRange::new(0, end_ts_i32));
            },
            _ => {
                time_intervals.push(FileTimeRange::new(start_ts_i32, MAX_INDEX_SAMPLES));
                for _i in 2..range_count {   
                    time_intervals.push(FileTimeRange::new(0, MAX_INDEX_SAMPLES));
                }
                time_intervals.push(FileTimeRange::new(0, end_ts_i32));
            }
        }

        // We have at least one file create the Object
        if file_index_vec.len() >= 1 {
            data_locator_vec = file_index_vec.into_iter()
                                             .map(|item| DataLocator::new(item.0, item.1, time_intervals[item.2.0], item.2.1))
                                             .collect();
            debug!("[READ] Returning Object {:?} ", data_locator_vec); 
            return Some(data_locator_vec);
        }
        None
    }
}

/// Returns a Vector of array of time intervals (in seconds) for the interval of time
fn time_intervals(start_time: i64, end_time: i64) -> Vec<[i32; 2]> {
    let mut time_intervals = Vec::new();
    let start_date = DateTime::<Utc>::from_utc(
                                            chrono::NaiveDateTime::from_timestamp_opt((start_time/1000).into(), 0).unwrap(),
                                              Utc,
                                                    );
    let end_date = DateTime::<Utc>::from_utc(
                                          chrono::NaiveDateTime::from_timestamp_opt((end_time/1000).into(), 0).unwrap(),
                                            Utc,
                                                    );
    let start_ts_i32 = day_elapsed_seconds(start_time);
    let end_ts_i32 = day_elapsed_seconds(end_time);
    let date_spread_size = DateRange(start_date, end_date).into_iter().count();
    match date_spread_size {
        1 => { time_intervals.push([start_ts_i32, end_ts_i32]);},
        2 => {
            time_intervals.push([start_ts_i32, MAX_INDEX_SAMPLES]);
            time_intervals.push([0, end_ts_i32]);
        },
        _ => {
            time_intervals.push([start_ts_i32, MAX_INDEX_SAMPLES]);
            for _i in 2..date_spread_size {   
                time_intervals.push([0, MAX_INDEX_SAMPLES]);
            }
            time_intervals.push([0, end_ts_i32]);
        }
    }
    time_intervals
}

/// Given a metric name and a time interval, returns all the files handles for the files that contain that data
pub fn get_file_index_time(metric_name: &str, start_time: i64, end_time: i64) -> Option<Vec<DataLocator>> {
    DataLocator::get_locators_for_range(metric_name, start_time, end_time)
}

pub fn data_locator_into_prom_data_point(data: Vec<DataLocator>) -> Vec<PromDataPoint> {
    debug!("[READ] Locators: {:?}", data);
    let mut data_points = Vec::new();
    for dl in data {
        let mut proms = dl.into_prom_data_point();
        if proms.len() > 0 { data_points.append(&mut proms); }
    }
    data_points
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
    // Files might not match the intervals of time, a time array of time intervals need to be done.
    
    

    // Where the samples land in the indexes
    let mut samples_locations: [i32; 2];
    for pack in file_vec.into_iter().enumerate() { 
        let iter_index = pack.0;
        let file = pack.1.0;
        let vsri = pack.1.1;
        debug!("[READ] Locating samples. VSRI {:?} TS: {} - {}",vsri, start_ts_i32, end_ts_i32);
        // Check if the timestamps intercept the index space
        if file_count == 1 { 
            debug!("[READ] Processing single file...");
            // Case 2
            // get_sample can return None
            if vsri.min() > end_ts_i32 || vsri.max() < start_ts_i32 { 
                debug!("[READ] No intersection. Returning.");
                return data_points; 
            }
            let start_sample = vsri.get_this_or_next(start_ts_i32);
            if start_sample.is_none() {
                // No sample in the file fits the current requested interval
                debug!("[READ] No intersection (Part2). Returning.");
                return data_points;
            }
            // If I can start reading the file, I can get at least one sample, so it is safe to unwrap.
            let end_sample = vsri.get_this_or_previous(end_ts_i32).unwrap();
            samples_locations = [start_sample.unwrap(), end_sample];
        } else {
        // Case 1
            debug!("[READ] Processing multiple files...");
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
        let flac_metric = SimpleFlacReader::new(file, start_time);
        let tmp_vec = vsri.get_all_timestamps();
        let start = samples_locations[0];
        let end = samples_locations[1]-1;
        debug!("[READ] Samples located! From {} to {}. TS available: {}",start, end, tmp_vec.len());
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
            Err(err) => {error!("[READ] Error processing FLaC file {:?}", err); continue;}
        }
    }
    debug!("[READ] Returning datapoints: {:?}", data_points);
    data_points
}