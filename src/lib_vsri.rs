/// Very Small Rolo Index
/// This is an index made for detection of gaps in continuous data with the same sampling rate.
/// Each continous segment of data will be mapped to a line using the formula y = mx + B plus
/// the number of points in the data series.
/// m - Sampling rate
/// b - Series initial point in time in [x,y]
/// x - sample # in the data file, this is ALWAYS sequential. There are no holes in samples
/// y - time
/// 
/// This way, discovering the segment number is solving the above equation for X if the 
/// time provided is bigger than the initial point.
/// 
/// best case for sample retrival O(1)
/// worst case O(N) (N is the number of segments)
/// Space usage: 5Bytes for 64k samples. 
/// Or: 30Bytes for 2^32 Samples
/// 
/// Example of content of an index
///     up_localhost:9090_2023-07-05
///     55745
///     59435
///     15,0,55745,166
///     15,166,58505,63

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use chrono::{DateTime, Utc, Timelike};

// TODO: This should be configurable. Indexes are build for 1 day worth of samples, at 1 sample per second
pub static MAX_INDEX_SAMPLES: i32 = 86400;

// Helper functions, this should be moved somewhere
/// Returns the number of seconds elapsed for the day provided in the `timestamp_sec`
pub fn day_elapsed_seconds(timestamp_sec: i64) -> i32 {
    let datetime = DateTime::<Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp_opt(timestamp_sec, 0).unwrap(),
        Utc,
    );
    // Extract the time components (hour, minute, and second) from the DateTime
    let hour= datetime.time().hour();
    let minute = datetime.time().minute();
    let second =  datetime.time().second();
    // Calculate the total seconds since the start of the day
    (hour * 3600 + minute * 60 + second) as i32
}

/// Returns the timestamp for the begining of the day given a DateTime object.
pub fn start_day_ts(dt: DateTime<Utc>) -> i64 {
    let hour= dt.time().hour();
    let minute = dt.time().minute();
    let second =  dt.time().second();
    dt.timestamp() - (hour * 3600 + minute * 60 + second) as i64
}

/// In this implementation we are writting sample by sample to the WAV file, so
/// we can't do a proper segment calculation. So there will a special first segment
/// that will hold the first point so we can calculate the segments from there.
/// 
/// # Examples
/// Creating a new index, metric is of expected time 0, but for sure location of X is 0
/// ```no_run
/// let vsri = VSRI::new("metric_name", 0, 0);
/// vsri.flush();
/// ```
/// Updating an index, adding point at time 5sec
/// ```no_run
/// let vsri = VSRI::load("metric_name").unwrap().update_for_point(5);
/// vsri.flush();
/// ```
/// Fetch a sample location from the index given a timestamp
/// ```no_run
/// let vsri = VSRI::load("metric_name").unwrap();
/// vsri.get_sample_location("metric_name", 5);
/// ```

/// Index Structure
/// index_name: Name of the index file we are indexing
/// min_ts: the minimum TS available in this file
/// max_ts: the highest TS available in this file
/// vsri_segments: Description of each segment
///                [sample_rate (m), initial_point(x,y), # of samples(lenght)]
/// Each segments describes a line with the form of mX + B that has a lenght 
/// of # of samples.
#[derive(Debug)]
pub struct VSRI {
    index_file: String,
    min_ts: i32,
    max_ts: i32,
    // TODO: ENUM here to make it simpler to understand what each point in the array means
    vsri_segments: Vec<[i32; 4]> // [Sample Rate (m), X0, Y0, # of Samples]
}

impl VSRI {

    /// Creates the index, it doesn't create the file in the disk
    /// flush needs to be called for that
    pub fn new(filename: &String) -> Self {
        println!("[DEBUG][INDEX] Creating new index!");
        let segments: Vec<[i32; 4]> = Vec::new();
        VSRI {
            index_file: filename.to_string(),
            min_ts: 0,
            max_ts: 0,
            vsri_segments: segments
            }
    }

    /// Given a filename and a time location, returns the sample location in the 
    /// data file. Or None in case it doesn't exist.
    pub fn get_sample_location(filename: String, y: i32) -> Option<i32> {
        let vsri = match  VSRI::load(&filename) {
            Ok(vsri) => vsri,
            Err(_err) => { return None }
         };
         if vsri.min() <= y && y <= vsri.max() {
            return vsri.get_sample(y)
         }
         None
    }

    /// Get the sample for this timestamp or the next one
    pub fn get_this_or_next(&self, y: i32) -> Option<i32> {
        let r = self.get_sample(y).or(self.get_next_sample(y));
        println!("[DEBUG][INDEX]This or next location {:?} for TS {}", r, y);
        r
    }

    /// Get the sample for this timestamp or the previous one
    pub fn get_this_or_previous(&self, y: i32) -> Option<i32> {
        let r = self.get_sample(y).or(self.get_previous_sample(y));
        println!("[DEBUG][INDEX]This or previous location {:?} for TS {}", r, y);
        r
    }

    /// Returns the next sample for the provided timestamp.
    /// This might be useful to find the next segment timestamp if the timestamp
    /// is in between segments. It will return None in case the timestamp is over
    /// the maximum timestamp of the index.
    pub fn get_next_sample(&self, y: i32) -> Option<i32> {
        if y < self.min() {
            return Some(0);
        } else if y >= self.max() {
            return None;
        }
        // It wasn't smaller, so let's see if we have a sample that matches
        for segment in self.vsri_segments.clone().into_iter().rev() {
            let first_sample = segment[1];
            let y0 = segment[2];
            if y <= y0 {
                return Some(first_sample);
            }
        }
        None
    }

    /// Returns the previous sample for the provided timestamp.
    /// This might be useful to find the previous segment timestamp if the timestamp
    /// is in between segments. It will return None in case the timestamp is bellow
    /// the minimum timestamp of the index.
    pub fn get_previous_sample(&self, y: i32) -> Option<i32> {
        if y < self.min() {
            return None;
        } else if y >= self.max() {
            // Return the last segment, # of samples. That is the total # of samples in a file 
            return Some(self.get_sample_count());
        }
        // Cycle through the segments
        for segment in &self.vsri_segments {
            let first_sample = segment[1];
            let y0 = segment[2];
            if y < y0 {
                // Return the last sample of the previous segment
                return Some(first_sample-1);
            }
        }
        None
    }

    /// Checks if the time segment provided falls in an empty space (Between 2 segments)
    /// This is useful to check intersections. If this function returns false the provided
    /// time segment does overlap with the existing time segments in the file
    pub fn is_empty(&self, time_segment: [i32; 2]) -> bool {
        // I could simple try to get 2 samples and if one of the returns, it is not empty
        // but I would walk segments twice instead of once
        match &self.vsri_segments.len() {
            1 => {
                // It starts or ends inside the segment (might be a single sample)
                if (time_segment[0] >= self.min() && time_segment[0] <= self.max()) ||
                   (time_segment[1] <= self.max() && time_segment[1] >= self.min()) {
                    return false;
                }
                // Or it contains the whole segment
                if time_segment[0] < self.min() && time_segment[1] > self.max() {
                    return false;
                }
            },
            _ => {
                // More than 1 segment
                let mut previous_seg_end: i32 = 0;
                let mut segment_count = 0;
                for segment in &self.vsri_segments {
                    let sample_rate = segment[0];
                    let y0 = segment[2];
                    let num_samples = segment[3];
                    let segment_end_y = y0 + (sample_rate * (num_samples - 1));
                    // If we are in the 2+ segment, lets test if the time falls in the middle
                    if segment_count >= 1 && 
                        (time_segment[0] > previous_seg_end && time_segment[1] < y0) {
                            return true;
                        }
                    // Could this be simplified with Karnaugh map? I'll dig my books later
                    // It starts or ends inside the segment
                    if (time_segment[0] >= y0 && time_segment[0] < segment_end_y) ||
                       (time_segment[1] < segment_end_y && time_segment[1] >= y0) {
                        return false;
                    }
                    // Or it contains the whole segment
                    if time_segment[0] < y0 && time_segment[1] > segment_end_y {
                        return false;
                    }
                    // At this point, time segments doesn't touch this segment.
                    previous_seg_end = segment_end_y;
                    segment_count += 1;
                }
            }
        }
        // Didn't find any intersection, or left in the middle, it is empty
        true
    }

    /// Update the index for the provided point
    /// y - time in seconds
    /// TODO: Change PANIC for proper error control
    pub fn update_for_point(&mut self, y: i32) {
        // Y needs to be bigger that the current max_ts, otherwise we are appending a point in the past
        // TODO: Quantiles sends several metrics for the same time, how to handle it?
        if y < self.max_ts {
            panic!("[DEBUG][INDEX] Trying to index a point in the past: {}, provided point: {}",self.max_ts, y );
        }
        self.max_ts = y;
        let segment_count = self.vsri_segments.len();
        // Empty segments, create a new one, this is also a new index, update the timestamps
        if segment_count == 0 {
            self.min_ts = y;
            self.vsri_segments.push(self.create_fake_segment(y));
            return
        }
        if self.is_fake_segment() {
            // In the presence of a fake segment (where m is 0), and a new point, we are now
            // in a situation we can calculate a decent segment
            self.vsri_segments[segment_count-1] = self.generate_segment(y);
        } else {
            // Check ownership by the current segment
            if self.fits_segment(y) {
                // It fits, increase the sample count and it's done
                println!("[DEBUG][INDEX] Same segment, updating. TS: {}", y);
                self.vsri_segments[segment_count-1][3] += 1;
                return
            }
            // If it doesn't fit, create a new fake segment
            self.vsri_segments.push(self.create_fake_segment(y));
        }
    }

    /// Minimum time stamp
    pub fn min(&self) -> i32 {
        self.min_ts
    }

    /// Maximum time stamp
    pub fn max(&self) -> i32 {
        self.max_ts
    }

    fn calculate_b(&self, segment: &[i32; 4]) -> i32 {
        // b = y - mx
        let b = segment[2] - segment[0] * segment[1];
        b
    }

    /// Returns the most recent (the last) calculated segment
    fn current_segment(&self) -> [i32; 4] {
        match self.vsri_segments.len() {
            0 => [0,0,0,0],
            _ => self.vsri_segments[self.vsri_segments.len()-1]
        }
    }

    /// Get the sample location for a given point in time, or None if there is no sample for that specific TS
    pub fn get_sample(&self, y: i32) -> Option<i32> {
        for segment in &self.vsri_segments {
            let sample_rate = segment[0];
            let y0 = segment[2];
            let num_samples = segment[3];

            let segment_end_y = y0 + (sample_rate * (num_samples - 1));

            if y >= y0 && y <= segment_end_y {
                // x = (y - b)/ m
                // TODO: This can return floats!
                let x_value = (y-self.calculate_b(segment))/sample_rate;
                return Some(x_value);
            }
        }
        None // No matching segment found for the given Y value
    }

    /// For a given sample position, return the timestamp associated
    pub fn get_time(&self, x:i32) -> Option<i32> {
        match x {
            0 => { return Some(self.min()); },
            _ if x > self.get_sample_count() => { return None; },
            _ if x == self.get_sample_count() => { return Some(self.max()); },
            // it is somewhere in the middle
            _ => {
                // Find the segment where X fits
                for segment in &self.vsri_segments {
                    if x >= segment[1] && x < (segment[1] + segment[3]) {
                        // Belongs here! Return Segment TS + the TS interval * x
                        let y = segment[2] + segment[0]*x;
                        return Some(y);
                    }
                    continue;
                }
                None
            }
        }
    }

    /// Returns a vector will all the timestamps covered by this index
    pub fn get_all_timestamps(&self) -> Vec<i32> {
        let mut time_vec = Vec::new();
        for segment in &self.vsri_segments {
            let samples = segment[3]; // Range is EXCLUSIVE above
            let time_step = segment[0];
            let initial_ts = segment[2];
            let time_iter = (0..samples)
                                        .map(|f| (f * time_step)+initial_ts);
            time_vec.extend(time_iter);
        }
        time_vec
    }

    pub fn get_sample_count(&self) -> i32 {
        let last_segment = self.current_segment();
        last_segment[3] + last_segment[1]
    }

    /// Generates a segment from a point. It uses information stored in the segment
    /// to regenerate the same segment with the new point information. 
    fn generate_segment(&self, y: i32) -> [i32; 4] {
        // Retrieve the last segment
        let last_segment = self.current_segment();
        // double check for correctness
        if last_segment[0] != 0 {
            return last_segment;
        }
        // Calculate the new segment
        // m = (y1-y0)/(x1-x0) -> (x1-x0) = 1 => m = y1-y0 (X is a sequence)
        let m = y-last_segment[2];
        // We got m, the initial points are the same, and now we have 2 samples
        [m, last_segment[1], last_segment[2], 2]
    }

    fn update_segment_samples(mut self) {
        let segment_count = self.vsri_segments.len();
        self.vsri_segments[segment_count-1][3] += 1;
    }

    /// Generate a fake segment, this can't be used for ownership testing
    /// x is the previous segment sample number
    /// We only have the first y0 point, nothing else
    fn create_fake_segment(&self, y:i32) -> [i32; 4] {
        println!("[DEBUG][INDEX] New segment, creating for point: {}", y);
        let segment = self.current_segment();
        // First point of the new segment: Prior starting point + Number of samples
        let x = segment[1] + segment[3];
        [0,x,y,1]
    }

    /// Checks if the most recent segment is a fake segment
    fn is_fake_segment(&self) -> bool {
        let last_segment = self.current_segment();
        last_segment[0] == 0
    }

    /// Returns true if a point fits the last segment of the index
    fn  fits_segment(&self, y: i32) -> bool {
        let last_segment = self.current_segment();
        let b = self.calculate_b(&last_segment);
        // What we have to check, is with the given y, calculate x.
        // Then check if x fits the interval for the current line
        // and it has to be the next one in the line
        // x = (y - b)/ m
        // TODO: Can return float, watch out
        let x_value = (y-b)/last_segment[0];
        println!("[DEBUG][INDEX] Fit Calculation (Segment {:?}). b: {},  x: {}, calculated x: {}",last_segment,b,(last_segment[3] + last_segment[1]),x_value);
        x_value == last_segment[3] + last_segment[1]
    }

    /// Writes the index to the disk
    /// File format
    /// line | content
    /// 1    | index name (should match file name). eg: cpu_index
    /// 2    | minimum timestamp on this file. eg: 10
    /// 3    | maximum timestamp on this file. eg: 34510 
    /// 4..N | Segments. 4 fields separated by commas. ex: 0,1,2,3
    pub fn flush(&self) -> Result<(), std::io::Error> {
        let file = File::create(format!("{}.vsri", &self.index_file))?;
        let mut writer = BufWriter::new(file);
    
        // Write index_file, min_ts, max_ts on the first three lines
        writeln!(writer, "{}", self.index_file)?;
        writeln!(writer, "{}", self.min_ts)?;
        writeln!(writer, "{}", self.max_ts)?;
    
        // Write each vsri_segment on a separate line
        for segment in &self.vsri_segments {
            writeln!(writer, "{},{},{},{}", segment[0], segment[1], segment[2], segment[3])?;
        }
    
        writer.flush()?;
        Ok(())
    }

    /// Reads an index file and loads the content into the structure
    /// TODO: Add error control (Unwrap hell)
    pub fn load(filename: &String) -> Result<Self, std::io::Error> {
        println!("[DEBUG][INDEX] Load existing index");
        let file = File::open(format!("{}.vsri", &filename))?;
        let reader = BufReader::new(file);
        let mut min_ts = 0;
        let mut max_ts = 0;
        let mut segments: Vec<[i32; 4]> = Vec::new();
        let mut i = 1; // Line 1,2 and 3 are not segments.
        for line in reader.lines() {
            let line = line?;
            match i {
                1 => {println!("[DEBUG][INDEX] Processing index file: {}", line)}
                2 => {min_ts = line.trim().parse::<i32>().unwrap();}
                3 => {max_ts = line.trim().parse::<i32>().unwrap();}
                _ => {
                    let values = line.split(',').map(|value| value.trim().parse::<i32>()).collect::<Result<Vec<i32>, _>>().unwrap();
                    segments.push([values[0],values[1],values[2],values[3]]);
                }
            }
            i+=1;
        }
        Ok(VSRI {
            index_file: filename.to_string(),
            min_ts,
            max_ts,
            vsri_segments: segments
            })
        }
    
}

