/// Very Small Rolo Index
/// This is an index made for detection of gaps in continuous data with the same sampling rate.
/// Each continous segment of data will be mapped to a line using the formula y = mx + B plus
/// the number of points in the data series.
/// m - Sampling rate
/// b - Series initial point in time in [x,y]
/// x - sample # in the data file
/// y - time
/// 
/// This way, discovering the segment number is solving the above equation for X if the 
/// time provided is bigger than the initial point.
/// 
/// best case for sample retrival O(1)
/// worst case O(N) (N is the number of segments)
/// Space usage: 5Bytes for 64k samples. 
/// Or: 30Bytes for 2^32 Samples

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};


/// 
/// In this implementation we are writting sample by sample to the WAV file, so
/// we can't do a proper segment calculation. So there will a special first segment
/// that will hold the first point so we can calculate the segments from there.
/// 
/// Creating a new index, metric is of expected time 0, but for sure location of X is 0
/// ```
/// VSRI::new("metric_name", 0, 0);
/// ```
/// Updating an index, adding point 1 at time 5sec
/// ```
/// VSRI::load("metric_name").unwrap().update_for_point(1,5);
/// ```
/// Fetch a point from the index
/// ```
/// VSRI::get_sample_location("metric_name", 5);
/// ```

/// Index Structure
/// index_name: Name of the index file we are indexing
/// min_ts: the minimum TS available in this file
/// max_ts: the highest TS available in this file
/// vsri_segments: Description of each segment
///                [sample_rate (m), initial_point(x,y), # of samples(lenght)]
/// Each segments describes a line with the form of mX + B that has a lenght 
/// of # of samples.
struct VSRI {
    index_file: String,
    min_ts: i32,
    max_ts: i32,
    vsri_segments: Vec<[i32; 4]> // [Sample Rate (m), X0, Y0, # of Samples]
}

impl VSRI {
    /// Reads an index file and loads the content into the structure
    /// TODO: Add error control
    pub fn load(filename: String) -> Result<VSRI, std::io::Error> {
        let file = File::open(&filename)?;
        let reader = BufReader::new(file);
        let mut segments: Vec<[i32; 4]> = Vec::new();
        let mut max_ts: u32 = 0;
        for line in reader.lines() {
            let line = line?;
            // Why did I do this?? I'm having an hard time figuring this out the next day...
            let values = line.split(',').map(|value| value.trim().parse::<i32>()).collect::<Result<Vec<i32>, _>>().unwrap();
            segments.push([values[0],values[1],values[2],values[3]]);
        }
        // Min TS is the initial point of the first segment
        let min_ts = segments[0][1];
        // Max TS is the initial point of the last segment plus the sample rate x number of samples
        let max_ts = segments[segments.len()-1][1] + segments[segments.len()-1][0] * (segments[segments.len()-1][2]-1);
        Ok(VSRI {
            index_file: filename,
            min_ts,
            max_ts,
            vsri_segments: segments
            })
    }

    /// Creates the index, it doesn't create the file in the disk
    /// flush needs to be called for that
    pub fn new(filename: String, x: i32, y: i32) -> VSRI {
        let mut segments: Vec<[i32; 4]> = Vec::new();
        segments.push([0,x,y,1]);
        VSRI {
            index_file: filename,
            min_ts: y,
            max_ts: y,
            vsri_segments: segments
            }
    }

    /// Given a filename and a time location, returns the sample location in the 
    /// data file. Or None in case it doesn't exist.
    pub fn get_sample_location(filename: String, y: i32) -> Option<i32> {
        let vsri = match  VSRI::load(filename) {
            Ok(vsri) => vsri,
            Err(_err) => { return None }
         };
         vsri.get_sample(y)
    }

    /// Update the index for the provided point
    /// x - segment position
    /// y - time in seconds
    /// TODO: Change PANIC for proper error control
    pub fn update_for_point(mut self, x: i32, y: i32) {
        // Y needs to be bigger that the current max_ts, otherwise we are appending a point in the past
        if y <= self.max_ts {
            panic!("[DEBUG] Trying to index a point in the past, or the same point! last point: {}, provided point: {}",self.max_ts, y );
        }
        self.max_ts = y;
        let segment_count = self.vsri_segments.len();
        if self.is_fake_segment() {
            // In the presence of a fake segment (where m is 0), and a new point, we are now
            // in a situation we can calculate a decent segment
            self.vsri_segments[segment_count-1] = self.generate_segment(x, y);
        } else {
            // Check ownership by the current segment
            if self.fits_segment(x, y) {
                // It fits, increase the sample count and it's done
                self.vsri_segments[segment_count-1][3] += 1;
                return
            }
            // If it doesn't fit, create a new fake segment
            self.vsri_segments.push(VSRI::create_fake_segment(x, y));
        }
    }

    /// Minimum time stamp
    pub fn min(self) -> i32 {
        self.min_ts
    }

    /// Maximum time stamp
    pub fn max(self) -> i32 {
        self.max_ts
    }

    fn calculate_b(&self, segment: &[i32; 4]) -> i32 {
        // b = y - mx
        let b = segment[2] - segment[0] * segment[1];
        b
    }

    /// Returns the most recent calculated segment
    fn current_segment(&self) -> [i32; 4] {
        self.vsri_segments[self.vsri_segments.len()-1]
    }

    /// Get the sample location for a given point in time, or None if there is no sample
    pub fn get_sample(&self, y: i32) -> Option<i32> {
        for segment in &self.vsri_segments {
            let sample_rate = segment[0];
            let y0 = segment[2];
            let num_samples = segment[3];

            let segment_end_y = y0 + (sample_rate * (num_samples - 1));

            if y >= y0 && y <= segment_end_y {
                // x = (y - b)/ m
                let x_value = (y-self.calculate_b(segment))/sample_rate;
                return Some(x_value);
            }
        }

        None // No matching segment found for the given Y value
    }

    /// Generates a segment from a point. It uses information stored in the segment
    /// to regenerate the same segment with the new point information. 
    fn generate_segment(&self, x: i32, y: i32) -> [i32; 4] {
        // Retrieve the last segment
        let last_segment = self.current_segment();
        // double check for correctness
        if last_segment[0] != 0 {
            return last_segment;
        }
        // Calculate the new segment
        // m = (y1-y0)/(x1-x0)
        let m = (y-last_segment[2])/(x-last_segment[1]);
        // We got m, the initial points are the same, and now we have 2 samples
        [m, last_segment[1], last_segment[2], 2]
    }

    fn update_segment_samples(mut self) {
        let segment_count = self.vsri_segments.len();
        self.vsri_segments[segment_count-1][3] += 1;
    }

    /// Generate a fake segment, this can't be used for ownership testing
    /// We only have the first x0,y0 point, nothing else
    fn create_fake_segment(x:i32, y:i32) -> [i32; 4] {
        [0,x,y,1]
    }

    /// Checks if the most recent segment is a fake segment
    fn is_fake_segment(&self) -> bool {
        let last_segment = self.current_segment();
        last_segment[0] == 0
    }

    /// Returns true if a point fits the last segment of the index
    fn fits_segment(&self, x: i32, y: i32) -> bool {
        let last_segment = self.current_segment();
        let b = self.calculate_b(&last_segment);
        let y1 = last_segment[0] * x - b;
        y == y1
    }

    /// Writes the index to the disk
    pub fn flush(&self) -> Result<(), std::io::Error> {
        let file = File::create(&self.index_file)?;
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
    
}

