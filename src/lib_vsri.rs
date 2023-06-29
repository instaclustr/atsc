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
use std::io::{BufRead, BufReader};

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

    /// Get the sample location for a given point in time, or None if there is no sample
    pub fn get_sample(&self, y: i32) -> Option<i32> {
        for segment in &self.vsri_segments {
            let sample_rate = segment[0];
            let x0 = segment[1];
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

    /// Generates a segment from a sample rate, points
    fn generate_segment() -> [i32; 4] {
        [0, 0, 0, 0]
    }

    /// Returns true if a point fits the last segment of the index
    fn fits_segment(self, x: i32, y: i32) -> bool {
        let last_segment = self.vsri_segments[self.vsri_segments.len()-1];
        let b = self.calculate_b(&last_segment);
        let y1 = last_segment[0] * x - b;
        y == y1
    }

    /// Writes the index to the disk
    pub fn flush(self) -> Result<(), std::io::Error> {
        Ok(())
    }
    
}