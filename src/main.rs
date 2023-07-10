mod wav_writer;
mod fs_utils;
mod lib_vsri;
mod flac_reader;
use fs_utils::{get_data_between_timestamps, data_locator_into_prom_data_point};
use wav_writer::WavMetric;

use async_trait::async_trait;
use std::{convert::Infallible, sync::Arc};

use prom_remote_api::{
    types::{
        Error, Label, Query, QueryResult, RemoteStorage, Result, Sample, TimeSeries, WriteRequest, MetricMetadata,
    },
    web,
};
use warp::Filter;
use std::fs::File;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};
use symphonia::core::io::MediaSourceStream;

use chrono::{DateTime, Utc, Timelike};

use crate::fs_utils::get_file_index_time;

// Data sampling frequency. How many seconds between each sample.
static DATA_INTERVAL_SEC: u32 = 1;
static DATA_INTERVAL_MSEC: i64 = 1000;
static FLAC_SAMPLE_RATE: u32 = 8000;

// THIS IS A HACK!! This is to fix the issue that we don't have the full day of samples.
static DELTA_SHIFT: u64 = 37696;

// HACK 2! CPU was adjusted to fit into the WAV file, needs to be divided by 100.
fn adjust_cpu(cpu: i16)-> i16 {
    return cpu/100
}

/// The rate at which the samples are added to the file, never match the sample rate of the flac file.
/// The way the enconder/decoder works an high enough sample rate is needed (8kHz minimun)
/// But we never retrieve metric data at such a high rate, so we need to convert between sample rates
/// so we can seek to the proper place.
fn get_flac_timeshift(real_time: i64) -> Time {
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
    println!("Seconds since start of the day: {}", seconds_today);
    // APPLYING THE HACK! Fix it for now
    seconds_today -= DELTA_SHIFT;
    // Now, shift it!
    let shifted_nanoseconds: u64 = (seconds_today*1_000_000_000)/(FLAC_SAMPLE_RATE * DATA_INTERVAL_SEC) as u64;
    let shifted_seconds = shifted_nanoseconds / 1_000_000_000; // Divide by 1 billion to get the number of seconds
    let shifted_remainder = (shifted_nanoseconds % 1_000_000_000) as u32; // Use modulus operator to get the remaining nanoseconds
    let time_object = Time::from_ss(shifted_seconds as u8, shifted_remainder).unwrap();
    println!("Shifted time: {} {}", time_object.seconds, time_object.frac);
    return time_object;
}

/// Get the path to the flac file that matches the real time.
fn get_flac_file_path(real_time: i64) -> String {
    // Time is in ms, convert it to seconds
    let datetime = DateTime::<Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp_opt(real_time/ 1000, 0).unwrap(),
        Utc,
    );
    // Transform datetime to string with the format YYYY-MM-DD
    let datetime_str = datetime.format("%Y-%m-%d.flac").to_string();
    return datetime_str;
}

fn get_flac_samples(metric: &str, start_time: i64, end_time: i64)-> std::result::Result<Vec<i16>, SymphoniaError> {
    // Let's select a file acordingly to the time
    let file_path = format!("{}_{}", metric, get_flac_file_path(start_time));
    println!("File Path: {}", file_path);
    let file = Box::new(File::open(file_path).unwrap());
    let reader = MediaSourceStream::new(file, Default::default());

    let format_options = FormatOptions::default();
    let decoder_options = DecoderOptions::default();
    let metadata_opts: MetadataOptions = Default::default();

    // Lets probe
    let probed = symphonia::default::get_probe().format(Hint::new().mime_type("FLaC"), reader, &format_options, &metadata_opts).unwrap();
    let mut format_reader = probed.format;
    let track = format_reader.default_track().unwrap();
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &decoder_options).unwrap();

    let sample_rate = format_reader.tracks()[0].codec_params.sample_rate.unwrap();

    let seek_point = SeekTo::Time {
        time: get_flac_timeshift(start_time),
        track_id: Some(format_reader.tracks()[0].id) };

    let end_point_ts = TimeBase::new(1, sample_rate).calc_timestamp(get_flac_timeshift(end_time));
    
    // Prepare to store data, with Optimal Seek (less performance) this can be a static value, otherwise will stay like this
    // Listen to me! This is messed up, and I'm cutting and trimming and messing up the buffer, but until I
    // implement decent sample parsing, it is what it is.
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
                    // Create the i16 sample buffer.
                    sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                }
                if let Some(buf) = &mut sample_buf {
                    buf.copy_interleaved_ref(decoded);
                    //buf.copy_planar_ref(decoded);
                    println!("[DEBUG] Sample number: {} Packet Duration: {} Packet Timestamp: {}", buf.len(), packet.dur, packet.ts);
                    for  sample in buf.samples() {
                        
                        if metric[..3].eq("cpu") {
                            buffer.push(adjust_cpu(*sample));
                        } else {
                            buffer.push(*sample);
                        }
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

fn get_flac_samples_to_prom(metric: &str, source: &str, _job: &str, start_ms: i64, end_ms: i64, step_ms: i64) -> Vec<Sample> {
    // TODO: Count the number of samples for the given metric! -> Can be done with the Index alone \m/ \m/
    // TODO: Do not ignore Job!
    // TODO: Do not ignore Step!
    // Just for today, Step in the files is always 15sec, 15000 ms.
    let sample_step = (step_ms/15000) as usize;
    if step_ms == 0 {
        return vec![Sample {
            value: 1.0,
            timestamp: start_ms,
        }];
    }
    // Build the metric name
    let metric_name = format!("{}_{}",metric, source);
    let files_to_parse = get_file_index_time(&metric_name, start_ms, end_ms);
    if files_to_parse.is_none() {
        return vec![Sample {
            value: 1.0,
            timestamp: start_ms,
            }];
    }
    //let prom_vec = get_data_between_timestamps(start_ms, end_ms, files_to_parse.unwrap());
    let prom_vec = data_locator_into_prom_data_point(files_to_parse.unwrap());
    let prom_len = prom_vec.len();
    //println!("[DEBUG][MAIN] Prom data points: {:?}", prom_vec);
    println!("[DEBUG][MAIN] Requested Step: {:?} Proposed Steps: {:?}", step_ms, sample_step);
    // Convert into Samples and apply step_ms
    //let mut out = Vec::new();
    //let mut prev_sample_ts: i64 = 0; 
    /*
    for (i, pdp) in prom_vec.into_iter().enumerate() {
        if i == 0 {
            out.push(Sample{value: pdp.point, timestamp: pdp.time});
            prev_sample_ts = pdp.time;
            continue;
        }
        if pdp.time < prev_sample_ts + step_ms { continue; }
        out.push(Sample{value: pdp.point, timestamp: pdp.time});
        prev_sample_ts = pdp.time;
    }
    println!("[DEBUG][MAIN] Requested Step: {:?} Proposed Steps: {:?} Original len {:?} Final len {:?}", step_ms, sample_step, prom_len, out.len());
    
    out  */
    prom_vec.iter().map(|pdp| Sample{value: pdp.point, timestamp: pdp.time}).collect()
    //prom_vec.iter().step_by(sample_step).map(|pdp| Sample{value: pdp.point, timestamp: pdp.time}).collect()
    //let flac_content = get_flac_samples(metric, start_ms, end_ms).unwrap();
    // Flac reader is ignoring step returning way to many samples. So we have to deal with step here
    // Transforming the result into Samples
    //let step_size: usize = (step_ms/DATA_INTERVAL_MSEC).try_into().unwrap();
    //println!("[DEBUG] # of FLaC samples: {} Step size ms: {} Internal step: {}", flac_content.len(), step_ms, step_size);
    //flac_content.iter().step_by(step_size).enumerate().map(|(i, sample)| Sample{value: *sample as f64, timestamp: (start_ms + (i as i64)*step_ms) as i64}).collect()
}

fn parse_remote_write_request(timeseries: &TimeSeries, metadata: Option<&MetricMetadata>) -> Result<()> {
    println!("[DEBUG][WRITE] samples: {:?}", timeseries.samples);
    println!("[DEBUG][WRITE] labels: {:?}", timeseries.labels);
    
    let mut metric: Option<&str> = None;
    let mut source: Option<&str> = None;
    let mut job: Option<&str> = None;
    
    for label in &timeseries.labels {
        match label.name.as_str() {
            "__name__" => metric = Some(&label.value),
            "instance" => source = Some(&label.value),
            "job" => job = Some(&label.value),
            _ => ()
        }
    }
    
    if let (Some(metric), Some(source), Some(job)) = (metric, source, job) {
        // Not going to share state, flush it once you're done.
        let mut wav_metric = WavMetric::new(metric.to_string(), source.to_string(), job.to_string());
        let mut metric_data = timeseries.samples.iter().map(|x| (x.timestamp, x.value)).collect();
        let mutable_metric_data = &mut metric_data;
        wav_metric.add_bulk_timeseries(mutable_metric_data);
        let _ = wav_metric.flush();
    } else {
        println!("[DEBUG][WRITE] Missing metric or source");
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct FlacStorage;

impl FlacStorage {
    fn with_context() -> impl Filter<Extract = (u64,), Error = Infallible> + Clone {
        warp::any().map(|| 1)
    }
}

#[async_trait]
impl RemoteStorage for FlacStorage {
    type Err = Error;
    type Context = u64;
    // TODO: Figure out why the empty Results
    async fn write(&self, _ctx: Self::Context, req: WriteRequest) -> Result<()> {
        //println!("flac write, req:{req:?}");
        if req.metadata.is_empty() {
            for timeseries in req.timeseries {
                let _ = parse_remote_write_request(&timeseries, None);
                //break;
            }
        } else {
            for (timeseries, metadata) in req.timeseries.iter().zip(req.metadata.iter()) {
                let _ = parse_remote_write_request(timeseries, Some(metadata));
            }
        }
        Ok(())
    }

    async fn process_query(&self, _ctx: &Self::Context, query: Query) -> Result<QueryResult> {
        println!("[DEBUG][MAIN] flac read, req:{query:?}");
        let metric = &query.matchers[0].value;
        // TODO: Get these values from somewhere else
        let job = "flac-remote";
        let instance = "localhost:9090";
        Ok(QueryResult {
            timeseries: vec![TimeSeries {
                labels: vec![
                    Label {
                        name: "job".to_string(),
                        value: job.to_string(),
                    },
                    Label {
                        name: "instance".to_string(),
                        value: instance.to_string(),
                    },
                    Label {
                        name: "__name__".to_string(),
                        value: metric.to_string(),
                    },
                ],
                samples: get_flac_samples_to_prom(
                    metric,
                    instance,
                    job,
                    query.start_timestamp_ms,
                    query.end_timestamp_ms,
                    query
                        .hints
                        .as_ref()
                        .map(|hint| hint.step_ms)
                        .unwrap_or(1000),
                ),
                ..Default::default()
            }],
        })
    }
}

#[tokio::main(flavor = "current_thread")]
// BIG TODO: Make the code configurable (loads of hardcoded stuff)
async fn main() {
    let storage = Arc::new(FlacStorage);
    let write_api = warp::path!("write")
        .and(web::warp::with_remote_storage(storage.clone()))
        .and(FlacStorage::with_context())
        .and(web::warp::protobuf_body())
        .and_then(web::warp::write);
    let query_api = warp::path!("read")
        .and(web::warp::with_remote_storage(storage))
        .and(FlacStorage::with_context())
        .and(web::warp::protobuf_body())
        .and_then(web::warp::read);

    let routes = warp::path("api").and(write_api.or(query_api));

    let port = 9201;
    println!("Listen on {port}...");
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}