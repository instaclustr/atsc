// Lucas - Once the project is far enough along I strongly reccomend reenabling dead code checks
#![allow(dead_code)]

mod flac_reader;
mod fs_utils;
mod lib_vsri;
mod wav_writer;
use fs_utils::data_locator_into_prom_data_point;
use wav_writer::WavMetric;

use async_trait::async_trait;
use std::{convert::Infallible, sync::Arc};

use prom_remote_api::{
    types::{
        Error, Label, MetricMetadata, Query, QueryResult, RemoteStorage, Result, Sample,
        TimeSeries, WriteRequest,
    },
    web,
};
use warp::Filter;

use log::{debug, error, info, warn};

#[macro_use]
extern crate log;

use crate::fs_utils::get_file_index_time;

// Data sampling frequency. How many seconds between each sample.
static VERSION: &str = "0.1.1";

fn get_flac_samples_to_prom(
    metric: &str,
    source: &str,
    _job: &str,
    start_ms: i64,
    end_ms: i64,
    step_ms: i64,
) -> Vec<Sample> {
    // TODO: #6 Count the number of samples for the given metric! -> Can be done with the Index alone \m/ \m/
    // TODO: #1 Do not ignore Job!
    // TODO: #2 Do not ignore Step!
    // Just for today, Step in the files is always 15sec, 15000 ms.
    let sample_step = (step_ms / 15000) as usize;
    if step_ms == 0 {
        return vec![Sample {
            value: 1.0,
            timestamp: start_ms,
        }];
    }
    // Build the metric name
    let metric_name = format!("{}_{}", metric, source);
    let files_to_parse = get_file_index_time(&metric_name, start_ms, end_ms);
    if files_to_parse.is_none() {
        error!("No data found!");
        return vec![Sample {
            value: 1.0,
            timestamp: start_ms,
        }];
    }
    //let prom_vec = get_data_between_timestamps(start_ms, end_ms, files_to_parse.unwrap());
    let prom_vec = data_locator_into_prom_data_point(files_to_parse.unwrap());
    let prom_len = prom_vec.len();
    //debug!("[MAIN] Prom data points: {:?}", prom_vec);
    debug!(
        "[MAIN] Returned samples: {:?} Requested Step: {:?} Proposed Steps: {:?}",
        prom_len, step_ms, sample_step
    );
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
    debug!("[MAIN] Requested Step: {:?} Proposed Steps: {:?} Original len {:?} Final len {:?}", step_ms, sample_step, prom_len, out.len());

    out  */
    prom_vec
        .iter()
        .map(|pdp| Sample {
            value: pdp.point,
            timestamp: pdp.time,
        })
        .collect()
    //prom_vec.iter().step_by(sample_step).map(|pdp| Sample{value: pdp.point, timestamp: pdp.time}).collect()
    //let flac_content = get_flac_samples(metric, start_ms, end_ms).unwrap();
    // Flac reader is ignoring step returning way to many samples. So we have to deal with step here
    // Transforming the result into Samples
    //let step_size: usize = (step_ms/DATA_INTERVAL_MSEC).try_into().unwrap();
    //debug!(" # of FLaC samples: {} Step size ms: {} Internal step: {}", flac_content.len(), step_ms, step_size);
    //flac_content.iter().step_by(step_size).enumerate().map(|(i, sample)| Sample{value: *sample as f64, timestamp: (start_ms + (i as i64)*step_ms) as i64}).collect()
}

fn parse_remote_write_request(
    timeseries: &TimeSeries,
    metadata: Option<&MetricMetadata>,
) -> Result<()> {
    debug!("[WRITE] samples: {:?}", timeseries.samples);
    debug!("[WRITE] labels: {:?}", timeseries.labels);
    debug!("[WRITE] metadata: {:?}", metadata);

    let mut metric: Option<&str> = None;
    let mut source: Option<&str> = None;
    let mut job: Option<&str> = None;

    for label in &timeseries.labels {
        match label.name.as_str() {
            "__name__" => metric = Some(&label.value),
            "instance" => source = Some(&label.value),
            "job" => job = Some(&label.value),
            _ => (),
        }
    }

    if let (Some(metric), Some(source), Some(job)) = (metric, source, job) {
        // Not going to share state, flush it once you're done.
        // TODO: #3 Improve write performance (?)
        let mut metric_data: Vec<(i64, f64)> = timeseries
            .samples
            .iter()
            .map(|x| (x.timestamp, x.value))
            .collect();
        if timeseries.samples.is_empty() {
            error!("[WRITE][MAIN] Empty samples: {:?}", timeseries.samples);
            return Ok(());
        }
        let mut wav_metric = WavMetric::new(
            metric.to_string(),
            source.to_string(),
            job.to_string(),
            metric_data[0].0,
        );
        let mutable_metric_data = &mut metric_data;
        wav_metric.add_bulk_timeseries(mutable_metric_data);
        match wav_metric.flush() {
            Ok(_) => return Ok(()),
            Err(_samples) => {
                // TODO: Improve this situation... (Retry?)
                return Ok(());
            }
        }
    } else {
        warn!("[WRITE] Missing metric or source");
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
        trace!("[MAIN][WRITE] req:{req:?}");
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
        debug!("[MAIN] flac read, req:{query:?}");
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
    env_logger::init();
    info!("FFT-Storage v. {}", VERSION);
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
    info!("Server up, listening on {} port {}", "127.0.0.1", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
