// --- Write layer
// Remote write spec: https://prometheus.io/docs/concepts/remote_write_spec/
struct WavMetric {
    metric_name: String,
    instance: String,
    job: String,
    timeseries_data: Option<Vec<WavData>>
}
// Here is where things get tricky. Either we have a single strutcure and implement several WavWriters or we segment at the metric collection level.
// The advantage of implementing at the writing level is that we can look into the data and make a better guess based on the data.
struct WavData {
    timestamp: i64,
    value: f64
}

impl WavMetric {
    pub fn new(name: String, source: String, job: String) -> WavMetric {
        WavMetric { metric_name: name, instance: source, job: job, timeseries_data: None }
    }
}