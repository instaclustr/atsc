/// All the utils/code related the to file management
/// 
/// Because I was trying to fix stuff within FLaC, and it needs to be sorted way before.
/// 
/// For a READ request that needs data for MetricX from Ta to Tn this would do the following:
/// 1. Do we have metricX? -> No, stop.
/// 2. Which file has Ta, and which has Tb?
///     2.1 Select them to read
/// 3. Do holes exist in between?
///     3.1 If yes, where?
///     3.2 What is the cost of having a timestamp in a separated channel? <- Probably going this way! Have a "tick counter" based on the data timestamp.
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
use std::io::{Read, Seek};
use std::mem;
use chrono::{DateTime, Utc, Duration, Timelike, Datelike};

static MAX_INDEX_SIZE:i32 = 86400;

struct DateRange(DateTime<Utc>, DateTime<Utc>);

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
struct DataPoint {
    actual_data: [u16; 4],
    time: u64,
}

/// This will return a data point from a FLAC file for the provided point in time
fn read_data_point(file: &mut File) -> DataPoint {
    let mut data_point = DataPoint {
        actual_data: [0; 4],
        time: 0,
    };
    data_point
}

fn get_file_names(start_time: i64, end_time: i64) -> Vec<String> {
    let mut data_file_names = Vec::new();
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
        // TODO: Change dataX to the metric name
        let data_file_name = format!("dataX_{}_{}_{}", day, month, year);
        let _file = match  fs::File::open(data_file_name.clone()) {
            Ok(file) => file,
            Err(_err) => { println!("File {} doesn't exist, skipping", data_file_name); continue; }
         };
        data_file_names.push(data_file_name);
    }
    data_file_names
}

// TODO: We shouldn't open files twice. If we open the files at get_file_names(...) we should provide the file handles and work from there.
fn get_data_between_timestamps(start_time: u64, end_time: u64, data_file_names: &[String]) -> Vec<DataPoint> {
    let mut data_points = Vec::new();
    for data_file_name in data_file_names {
        let mut file = fs::File::open(data_file_name).unwrap();
        let mut current_time = 0;
        loop {
            let data_point = read_data_point(&mut file);
            current_time = data_point.time;
            if current_time >= start_time && current_time <= end_time {
                data_points.push(data_point);
            }
            if current_time > end_time {
                break;
            }
        }
    }
    data_points
}

/* TODO: I do need to learn how to do proper testing
fn main() {
    let start_time = 1655760000000;
    let end_time = 1655760500000;
    let data_file_names = get_file_names(start_time, end_time);
    let data_points = get_data_between_timestamps(start_time, end_time, &data_file_names);
    println!("{:?}", data_points);
}
 */