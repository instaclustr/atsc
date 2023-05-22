use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use prometheus::{Gauge, GaugeVec, Opts, Registry, TextEncoder, Encoder};

use warp::{http::StatusCode, Filter, Rejection, Reply};

use claxon::{FlacReader};

#[tokio::main]
async fn main() {
    let path = Path::new("2023-05-12_00-15-23.flac");
    let file = File::open(path).unwrap();

    let registry = Registry::new();

    // Get a vector with all the CPUs
    let mut cpus_vec = Vec::new();

    // Create a collector for the CPU usage metric.
    let cpu_gauge_opts = Opts::new(
        "cpu_usage",
        "The CPU usage of a specific CPU in percentage",
    ).const_label("host", "localhost");
    let cpu_gauge_vec = GaugeVec::new(cpu_gauge_opts, &["cpu"]).unwrap();

    for cpu in 0..8 {
        let cpu_usage_opts = Opts::new(
            &format!("cpu_usage_{}", cpu),
            &format!("The CPU usage of CPU {} in percentage.", cpu)
        ).const_label("host", "localhost");
    
        let cpu_usage = Gauge::with_opts(cpu_usage_opts).unwrap();
    
        cpu_gauge_vec.with_label_values(&[&cpu.to_string()]).set(0.0);
        registry.register(Box::new(cpu_usage.clone())).unwrap();
        cpus_vec.push(cpu_usage);
    }

    // Create a FLAC reader for the file.
    let mut reader = FlacReader::open(path).unwrap();

    // Read the CPU monitoring data from the file.
    let mut samples = Vec::new();
    let mut i=0;
    for sample in reader.samples() {
        if i==cpus_vec.len()  { i=0;}
        let sample = (sample.unwrap() as f32)/100.0;
        cpus_vec[i].add(sample.into());
        samples.push(sample);
        i+=1;
    }

    // Calculate the timestamps of the samples.
    let mut timestamps = Vec::new();
    let filename = path.file_name().unwrap().to_str().unwrap();
    let timestamp_str = &filename[..19];
    let timestamp = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d_%H-%M-%S").unwrap();
    // For each sample, calculate the timestamp and add it to the timestamps vector. There are samples for the same timestamp
    for _sample in 0..samples.len() {
        // add 500ms to the timestamp
        let timestamp = timestamp + chrono::Duration::milliseconds(500);
        for _cpu in 0..8 {
            timestamps.push(timestamp);
        }
    }
    // Publish the CPU monitoring data to Prometheus.
    let mut data_vec = Vec::new();
    for (cpu_value, timestamp) in samples.iter().zip(timestamps.iter()) {
        data_vec.push((timestamp, cpu_value));
    }

    let metrics_route = warp::path("metrics").map(move || {
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        warp::reply::with_header(buffer, "Content-Type", encoder.format_type())
    });

    let routes = metrics_route;
    let server = warp::serve(routes);
    server.run(([0, 0, 0, 0], 8080)).await;

}
