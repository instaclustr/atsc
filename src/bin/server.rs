/*
This will be a Prometheus compatible API server.
It will be able to expose metrics about the system,
the metrics will be stored in FLAC files. 

Each file will contain a single metric, the metric name is in the filename.
Also in the filename there will be a timestamp which indicates the start of the metric.

The data part of the file contains 8 16bit of CPU percentage values.
The data is stored in little endian format.
Also there is 8 values for each 500ms. Header is set for 8Hz.

So, how many samples is an minute of data?


 */

/* Read a WAV file,  */
fn read_metrics_from_wav(filename: &str) -> Vec<i16> {
    let mut reader = hound::WavReader::open(filename).unwrap();
    let num_samples = reader.len() as usize / reader.spec().channels as usize;
    let num_channels = reader.spec().channels as usize;
    
    // Create a vector to hold the audio data
    let mut samples = Vec::with_capacity(num_samples * num_channels);
    
    // Iterate over the samples and channels and push each sample to the vector
    for sample in reader.samples::<i16>() {
        samples.push(sample.unwrap());
    }
    return samples;
}

/* Read a FLAC file */
fn read_metrics_from_flac(filename: &str) -> Vec<i16> {
    let mut reader = claxon::FlacReader::open(filename).unwrap();
        // Create a vector to hold the audio data
    let mut samples = Vec::with_capacity(reader.streaminfo().samples.unwrap() as usize);
    for sample in reader.samples() {
        samples.push(sample.unwrap() as i16);
    }
    return samples;
}

fn main() {
    println!("Testing, does FLAC reading is the same as WAV?");
    let filename = "2023-05-11_15-11-19.wav";
    let filename_flac = "2023-05-11_15-11-19.flac";
    let samples = read_metrics_from_wav(filename);
    //println!("{:?}", samples);
    let samples_flac = read_metrics_from_flac(filename_flac);
    //println!("{:?}", samples_flac);
    assert_eq!(samples, samples_flac);
}