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
use chrono::{DateTime, Utc};

static MAX_INDEX_SIZE:i32 = 86400;

#[derive(Debug)]
struct DataPoint {
    actual_data: [u16; 4],
    time: u64,
}

fn read_data_point(file: &mut File) -> Result<DataPoint, std::io::Error> {
    let mut data_point = DataPoint {
        actual_data: [0; 4],
        time: 0,
    };
    file.read_exact(&mut data_point.actual_data)?;
    file.seek(std::io::SeekFrom::Current(2))?;
    file.read_exact(&mut data_point.time)?;
    Ok(data_point)
}

fn get_file_names(start_time: u64, end_time: u64) -> Vec<String> {
    let mut data_file_names = Vec::new();
    let start_date = DateTime::<Utc>::timestamp_millis_opt(start_time).unwrap();
    let end_date = Utc.timestamp(end_time).unwrap();
    for date in start_date..end_date {
        let day = date.day();
        let month = date.month();
        let year = date.year();
        let data_file_name = format!("dataX_{}_{}_{}", day, month, year);
        let mut file = fs::File::open(data_file_name).unwrap_or_else(|_| {
            // File doesn't exist, skip it
            println!("File {} doesn't exist, skipping", data_file_name);
            None
        });
        data_file_names.push(data_file_name);
    }
    data_file_names
}

fn get_data_between_timestamps(
    start_time: u64,
    end_time: u64,
    data_file_names: &[String],
) -> Vec<DataPoint> {
    let mut data_points = Vec::new();
    for data_file_name in data_file_names {
        let mut file = fs::File::open(data_file_name).unwrap();
        let mut current_time = 0;
        loop {
            let data_point = read_data_point(&mut file)?;
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

fn main() {
    let start_time = 1655760000000;
    let end_time = 1655760500000;
    let data_file_names = get_file_names(start_time, end_time);
    let data_points = get_data_between_timestamps(start_time, end_time, &data_file_names);
    println!("{:?}", data_points);
}