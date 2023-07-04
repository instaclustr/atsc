use std::fs::{File, self};
use std::time::{SystemTime, Duration};

use symphonia::core::audio::SampleBuffer;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};
use symphonia::core::io::MediaSourceStream;

use chrono::{DateTime, Utc, Timelike};

use crate::lib_vsri::VSRI;

// Data sampling frequency. How many seconds between each sample.
static DATA_INTERVAL_SEC: u32 = 1;
static DATA_INTERVAL_MSEC: i64 = 1000;
static FLAC_SAMPLE_RATE: u32 = 8000;

// --- Flac Reader
// Remote Reader Spec: ?

/* --- File Structure STRUCTURE
note: t=point in time, chan = channel, samples are the bytes for each channel.
      in this example, each sample is made of 2 bytes (16bit)
+---------------------------+---------------------------+-----
|         Frame 1           |         Frame 2           |  etc
+-------------+-------------+-------------+-------------+-----
| chan 1 @ t1 | chan 2 @ t1 | chan 1 @ t2 | chan 2 @ t2 |  etc
+------+------+------+------+------+------+------+------+-----
| byte | byte | byte | byte | byte | byte | byte | byte |  etc
+------+------+------+------+------+------+------+------+-----
 */ 

/// Structure that holds the samples for a metric that is in a FLaC file
/// It would only hold the number of samples needed for the provided interval
/// if the end of the interval is bigger than what is contained in the file,
///  the whole file content is returned.
 pub struct FlacMetric {
    metric_name: String,      // Metric name provided by prometheus
    instance: String,         // Instance name provided by prometheus
    job: String,              // Job name provided by prometheus 
    timeseries_data: Vec<(i64, f64)>, // Sample Data
    file_path: String,        // The filepath where the metric is
    interval_start: i64,      // The start interval in timestamp with miliseconds
    interval_end: i64         // The end of the interval in timestamp with miliseconds
}

impl FlacMetric {
    pub fn new(name: String, source: String, job: String, start_ts: i64, end_ts: i64) -> Self {
        // Creation time
        let time = FlacMetric::datetime_from_ms(start_ts);
        let file_name = format!("{}_{}_{}.flac", name, source, time);
        let file_path = format!("./{}", file_name);
        FlacMetric { metric_name: name,
                    instance: source,
                    job,
                    timeseries_data: Vec::new(),
                    file_path,
                    interval_start: start_ts,
                    interval_end: end_ts
                 }
    }

    fn datetime_from_ms(real_time: i64) -> String {
        // Time is in ms, convert it to seconds
        let datetime = DateTime::<Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(real_time/ 1000, 0).unwrap(),
            Utc,
        );
        // Transform datetime to string with the format YYYY-MM-DD
        let datetime_str = datetime.format("%Y-%m-%d").to_string();
        return datetime_str;
    }

    /// Load sample data into the Flac Object
    fn load_samples(self) -> Vec<(i64, f64)> {
        Vec::new()
    }

    /// The rate at which the samples are added to the file, never match the sample rate of the flac file.
    /// The way the enconder/decoder works an high enough sample rate is needed (8kHz minimun)
    /// But we never retrieve metric data at such a high rate, so we need to convert between sample rates
    /// so we can seek to the proper place.
    fn get_flac_timeshift(real_time: i64, shift: Option<u64>) -> Time {
        // real_time is ms since EPOCH, so it includes a timestamp in it
        // Convert the timestamp from milliseconds to seconds
        let timestamp_sec = real_time / 1000;
        // Convert the timestamp to a DateTime in UTC
        let datetime = DateTime::<Utc>::from_utc(
            chrono::NaiveDateTime::from_timestamp_opt(timestamp_sec, 0).unwrap(),
            Utc,
        );
        // Extract the time components (hour, minute, and second) from the DateTime
        let hour= datetime.time().hour();
        let minute = datetime.time().minute();
        let second =  datetime.time().second();
        // Calculate the total seconds since the start of the day
        let mut seconds_today: u64 = (hour * 3600 + minute * 60 + second).into();
        // If the file doesn't contain all the DAY in samples, we need to shift into the starting part of
        // the file
        seconds_today -= shift.unwrap_or(0);
        // Now, shift it!
        let shifted_nanoseconds: u64 = (seconds_today*1_000_000_000)/(FLAC_SAMPLE_RATE * DATA_INTERVAL_SEC) as u64;
        let shifted_seconds = shifted_nanoseconds / 1_000_000_000; // Divide by 1 billion to get the number of seconds
        let shifted_remainder = (shifted_nanoseconds % 1_000_000_000) as u32; // Use modulus operator to get the remaining nanoseconds
        let time_object = Time::from_ss(shifted_seconds as u8, shifted_remainder).unwrap();
        println!("Shifted time: {} {}", time_object.seconds, time_object.frac);
        return time_object;
    }


    /// Get the samples from an indexed file 
    fn get_indexed_flac_samples(file_path: &str, samples: Vec<i32>) -> std::result::Result<Vec<f64>, SymphoniaError> {
        
        let sample_vec: Vec<f64> = Vec::new();
        // Let's select a file acordingly to the time
        let file = Box::new(File::open(file_path).unwrap());
        // Get the file created time, to in case it is needed, apply a timeshift
        // TODO: Too many corner cases where this would go wrong, fix it later
        let metadata = fs::metadata(file_path)?;
        let time_shift = metadata.created().unwrap_or(SystemTime::now());
        let reader = MediaSourceStream::new(file, Default::default());
        let format_options = FormatOptions::default();
        let decoder_options = DecoderOptions::default();
        let metadata_opts: MetadataOptions = Default::default();
        // Probe to get file information, we hint that this is a FLaC file
        let probed = symphonia::default::get_probe().format(Hint::new().mime_type("FLaC"), reader, &format_options, &metadata_opts).unwrap();
        let mut format_reader = probed.format;
        let track = format_reader.default_track().unwrap();
        let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &decoder_options).unwrap();
        let channels = decoder.codec_params().channels.unwrap().count();
        // It should be a single track file
        let sample_rate = format_reader.tracks()[0].codec_params.sample_rate.unwrap();
        // Just to make it compile
        Ok(sample_vec)
    }

    /// Get the samples from the file provided within the interval provided
    /// TODO: Split the decoding loop into a separated function so it can deal with different Vec sizes
    fn get_flac_samples(file_path: &str, start_time: i64, end_time: i64) -> std::result::Result<Vec<f64>, SymphoniaError> {
        // Let's select a file acordingly to the time
        let file = Box::new(File::open(file_path).unwrap());

        // Get the file created time, to in case it is needed, apply a timeshift
        // TODO: Too many corner cases where this would go wrong, fix it later
        let metadata = fs::metadata(file_path)?;
        let time_shift = metadata.created().unwrap_or(SystemTime::now());
    
        let reader = MediaSourceStream::new(file, Default::default());

        let format_options = FormatOptions::default();
        let decoder_options = DecoderOptions::default();
        let metadata_opts: MetadataOptions = Default::default();

        // Probe to get file information, we hint that this is a FLaC file
        let probed = symphonia::default::get_probe().format(Hint::new().mime_type("FLaC"), reader, &format_options, &metadata_opts).unwrap();
        let mut format_reader = probed.format;
        let track = format_reader.default_track().unwrap();
        let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &decoder_options).unwrap();
        let channels = decoder.codec_params().channels.unwrap().count();
        // It should be a single track file
        let sample_rate = format_reader.tracks()[0].codec_params.sample_rate.unwrap();

        let seek_point = SeekTo::Time {
            time: FlacMetric::get_flac_timeshift(start_time, Some(0)),
            track_id: Some(format_reader.tracks()[0].id) };

        // TODO: Fix the timeshift if needed
        let end_point_ts = TimeBase::new(1,
                                            sample_rate)
                                            .calc_timestamp(FlacMetric::get_flac_timeshift(end_time,
                                                            Some(0)));
        
        let mut buffer = Vec::new();
        let mut sample_buf = None;
        // Seek to the correct point
        let initial_point = format_reader.seek(SeekMode::Accurate, seek_point);
        let point = match initial_point {
            Ok(point) => { 
                println!("Initial point: {:?}", point);
                point
            },
            Err(err) => { panic!("Unable to find starting point! Error: {}", err); }
        };
        loop {
            // Get the next packet from the media format.
            let packet = match format_reader.next_packet() {
                Ok(packet) => packet,
                Err(err) => {
                    // A unrecoverable error occured, halt decoding.
                    panic!("{}", err);
                }
            };
            // If we are above the end TS, stop!
            // In the best case we only added in full Packet of samples, so we have to trim the bufffer
            if packet.ts >= end_point_ts {
                let buff_total_size = end_point_ts - point.required_ts;
                if buffer.len() > buff_total_size as usize {
                    buffer.drain(buff_total_size as usize..);
                }
                println!("Packet TS : {:?}, End Point Time: {:?}", packet.ts, end_point_ts);
                break;
            }
            // Decode the packet into audio samples.
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Consume the decoded audio samples (see below).
                    if sample_buf.is_none() {
                        // Get the audio buffer specification.
                        let spec = *decoded.spec();
                        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                        let duration = decoded.capacity() as u64;
                        // Create the sample buffer.
                        sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                    }
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(decoded);
                        let mut i16_samples: [u16; 4] = [0,0,0,0];
                        let mut i = 1; // Starting at 1, channel number is not 0 indexed...
                        for  sample in buf.samples() {
                            if i >= channels {
                                buffer.push(FlacMetric::join_u16_into_f64(i16_samples));
                                i = 1;
                            }
                            i16_samples[i-1] = *sample as u16;
                            i += 1;
                        }
                        //print!("\rSamples decoded: {:?} samples", buffer);
                    }
                }
                Err(SymphoniaError::IoError(_)) => {
                    // The packet failed to decode due to an IO error, skip the packet.
                    continue;
                }
                Err(SymphoniaError::DecodeError(_)) => {
                    // The packet failed to decode due to invalid data, skip the packet.
                    continue;
                }
                Err(err) => {
                    // An unrecoverable error occured, halt decoding.
                    panic!("{}", err);
                }
            }
            // Trim initial uneeded samples
            if packet.ts < point.required_ts {
                let trim_size = (point.required_ts - packet.ts) as usize;
                buffer.drain(0..trim_size);
            }

        }
        Ok(buffer)
    }

    /// Recreate a f64
    fn join_u16_into_f64(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64) |
                ((bits[1] as u64) << 16) |
                ((bits[2] as u64) << 32) |
                ((bits[3] as u64) << 48);
    
    let f64_value = f64::from_bits(u64_bits);
    
    f64_value
    }
}