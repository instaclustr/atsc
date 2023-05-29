
/* Starting a changelog here

TODO:
Write metrics to FLAC files from Prometheus Write
Return the same metrics to prometheus via the remote read


## 26/05/2023
 - Currently Reading From Flac and serving prometheus via remote write

 */

use async_trait::async_trait;
use std::{convert::Infallible, sync::Arc};

use prom_remote_api::{
    types::{
        Error, Label, Query, QueryResult, RemoteStorage, Result, Sample, TimeSeries, WriteRequest,
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

fn extract_flac_content_from_interval(start_time: u64, end_time: u64)-> std::result::Result<Vec<i16>, SymphoniaError> {
    let file_path = "2023-05-11_15-11-19.flac";

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
        time: Time::new(start_time, 0.0),
        track_id: Some(format_reader.tracks()[0].id) };

    let end_point_ts = TimeBase::new(1, sample_rate).calc_timestamp(Time::new(end_time, 0.0));
    
    // Prepare to store data, with Optimal Seek (less performance) this can be a static value, otherwise will stay like this
    let mut buffer = Vec::new();
    let mut sample_buf = None;
    // Seek to the correct point
    let initial_point = format_reader.seek(SeekMode::Coarse, seek_point);
    match initial_point {
        Ok(point) => { println!("Initial point: {:?}", point);},
        Err(err) => { panic!("Unable to find starting point! Error: {}", err); }
    }
    
    // Not stopping on the required time (yet)
    loop {
        // Get the next packet from the media format.
        let packet = match format_reader.next_packet() {
            Ok(packet) => packet,
            Err(err) => {
                // A unrecoverable error occured, halt decoding.
                panic!("{}", err);
            }
        };
        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Consume the decoded audio samples (see below).
                if sample_buf.is_none() {
                    // Get the audio buffer specification.
                    let spec = *decoded.spec();
                    // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                    let duration = decoded.capacity() as u64;
                    // Create the f32 sample buffer.
                    sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                }
                if let Some(buf) = &mut sample_buf {
                    //buf.copy_interleaved_ref(decoded);
                    buf.copy_planar_ref(decoded);
                    for sample in buf.samples() {
                        buffer.push(*sample);
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
        if packet.ts >= end_point_ts {
            // Stop the loop, we are done!
            println!("Packet TS : {:?}, Packet Time: {:?}", packet.ts, end_point_ts);
            break;
        }
    }
    Ok(buffer)
}

fn get_flac_samples_to_prom(start_ms: i64, end_ms: i64, step_ms: i64) -> Vec<Sample> {
    if step_ms == 0 {
        return vec![Sample {
            value: 1.0,
            timestamp: start_ms,
        }];
    }
    // Ignoring the step for now
    // Start ms and end ms are Milliseconds since EPOCH, needs a conversion here.
    // For now, lets get a couple of samples at random
    //let flac_content = extract_flac_content_from_interval(start_ms as u64, end_ms as u64).unwrap();
    let flac_content = extract_flac_content_from_interval(3, 7).unwrap();
    // Transforming the result into Samples
    // It can only return has many results as (END - START / STEP)
    let return_samples_number = (end_ms - start_ms)/step_ms;
    println!("Returning {} samples out of {}", return_samples_number, flac_content.len());
    flac_content.iter().enumerate().map(|(i, sample)| Sample{value: *sample as f64, timestamp: (start_ms + (i as i64)*step_ms) as i64}).take(return_samples_number as usize).collect()
    
}

// For testing sake, I'm always sending the the same block of the FLAC file to the server on instant query,
// and the same sequence on range query

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

    async fn write(&self, _ctx: Self::Context, req: WriteRequest) -> Result<()> {
        //println!("flac write, req:{req:?}");
        Ok(())
    }

    async fn process_query(&self, _ctx: &Self::Context, query: Query) -> Result<QueryResult> {
        println!("flac read, req:{query:?}");

        Ok(QueryResult {
            timeseries: vec![TimeSeries {
                labels: vec![
                    Label {
                        name: "job".to_string(),
                        value: "flac-remote".to_string(),
                    },
                    Label {
                        name: "instance".to_string(),
                        value: "127.0.0.1:9201".to_string(),
                    },
                    Label {
                        name: "__name__".to_string(),
                        value: "up".to_string(),
                    },
                ],
                samples: get_flac_samples_to_prom(
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

#[derive(Clone, Copy)]
struct MockStorage;

fn generate_samples(start_ms: i64, end_ms: i64, step_ms: i64) -> Vec<Sample> {
    // instant query
    if step_ms == 0 {
        return vec![Sample {
            value: 1.0,
            timestamp: start_ms,
        }];
    }

    // range query
    (start_ms..end_ms)
        .step_by(step_ms as usize)
        .enumerate()
        .map(|(i, timestamp)| Sample {
            value: 1.0 + i as f64,
            timestamp,
        })
        .collect()
}
impl MockStorage {
    fn with_context() -> impl Filter<Extract = (u64,), Error = Infallible> + Clone {
        warp::any().map(|| 1)
    }
}

#[async_trait]
impl RemoteStorage for MockStorage {
    type Err = Error;
    type Context = u64;

    async fn write(&self, _ctx: Self::Context, req: WriteRequest) -> Result<()> {
        // Very verbose, for now...
        //println!("mock write, req:{req:?}");
        Ok(())
    }

    async fn process_query(&self, _ctx: &Self::Context, query: Query) -> Result<QueryResult> {
        println!("mock read, req:{query:?}");

        Ok(QueryResult {
            timeseries: vec![TimeSeries {
                labels: vec![
                    Label {
                        name: "job".to_string(),
                        value: "mock-remote".to_string(),
                    },
                    Label {
                        name: "instance".to_string(),
                        value: "127.0.0.1:9201".to_string(),
                    },
                    Label {
                        name: "__name__".to_string(),
                        value: "up".to_string(),
                    },
                ],
                samples: generate_samples(
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
async fn main() {
    //let storage = Arc::new(MockStorage);
    let storage = Arc::new(FlacStorage);
    let write_api = warp::path!("write")
        .and(web::warp::with_remote_storage(storage.clone()))
        .and(MockStorage::with_context())
        .and(web::warp::protobuf_body())
        .and_then(web::warp::write);
    let query_api = warp::path!("read")
        .and(web::warp::with_remote_storage(storage))
        .and(MockStorage::with_context())
        .and(web::warp::protobuf_body())
        .and_then(web::warp::read);

    let routes = warp::path("api").and(write_api.or(query_api));

    let port = 9201;
    println!("Listen on {port}...");
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}