/*
This file compares a FLAC and WAV file and if the content is identical
Also good to test if the FLAC and WAV read routines are good
*/

/* Read a WAV file,  */
fn _read_metrics_from_wav(filename: &str) -> Vec<i16> {
    let mut reader = hound::WavReader::open(filename).unwrap();
    let num_samples = reader.len() as usize / reader.spec().channels as usize;
    let num_channels = reader.spec().channels as usize;

    // Create a vector to hold the audio data
    let mut samples = Vec::with_capacity(num_samples * num_channels);

    // Iterate over the samples and channels and push each sample to the vector
    for sample in reader.samples::<i16>() {
        samples.push(sample.unwrap());
    }
    samples
}

/* Read a FLAC file */
fn read_metrics_from_flac(filename: &str) -> Vec<u16> {
    let mut reader = claxon::FlacReader::open(filename).unwrap();
    // Create a vector to hold the audio data
    let mut samples = Vec::with_capacity(reader.streaminfo().samples.unwrap() as usize);
    for sample in reader.samples() {
        samples.push(sample.unwrap() as u16);
    }
    samples
}

fn read_metrics_from_flac_by_bloc(filename: &str) -> Vec<u16> {
    let mut sample_vec: Vec<u16> = Vec::new();
    let mut reader = claxon::FlacReader::open(filename).unwrap();
    let channels = reader.streaminfo().channels as usize;
    let mut sample_channel_data = vec![0u16; channels];
    let mut frame_reader = reader.blocks();
    let mut block = claxon::Block::empty();

    loop {
        // Read a single frame. Recycle the buffer from the previous frame to
        // avoid allocations as much as possible.
        match frame_reader.read_next_or_eof(block.into_buffer()) {
            Ok(Some(next_block)) => block = next_block,
            Ok(None) => break, // EOF.
            Err(error) => panic!("[DEBUG][READ][FLAC] {}", error),
        }
        for sample in 0..block.duration() {
            #[allow(clippy::needless_range_loop)]
            for channel in 0..channels {
                sample_channel_data[channel] = block.sample(channel as u32, sample) as u16;
            }

            // Process the sample_channel_data as needed
            for &sample in &sample_channel_data {
                sample_vec.push(sample);
            }

            // Optionally, can print debug information
            println!(
                "Sample {}/{}, Channels: {:?}",
                sample,
                block.duration(),
                &sample_channel_data
            );
        }
    }
    sample_vec
}

fn _read_metrics_from_flac_in_interval(filename: &str, start: u32, end: u32) -> Vec<i16> {
    let mut reader = claxon::FlacReader::open(filename).unwrap();
    // Create a vector to hold the audio data
    let start_sample = start * reader.streaminfo().sample_rate;
    let end_sample = end * reader.streaminfo().sample_rate;
    //let mut samples = Vec::with_capacity(reader.streaminfo().samples.unwrap() as usize);
    let mut samples: Vec<i16> = Vec::new();
    for (i, sample) in reader.samples().enumerate() {
        let i = i as u32;
        if start_sample <= i && i <= end_sample {
            samples.push(sample.unwrap() as i16);
        } else if i > end_sample {
            break;
        }
    }
    samples
}

fn main() {
    println!("Testing, does FLAC reading is the same as WAV?");
    let _filename = "2023-05-11_15-11-19.wav";
    let filename_flac =
        "/home/crolo/code/prom_data/go_memstats_frees_total_localhost:9090_2023-07-07.flac";
    let _filename_flac_single = "3_single_channel.flac";
    //let samples = read_metrics_from_wav(filename);
    //println!("{:?}", samples);
    let samples_flac = read_metrics_from_flac(filename_flac);
    let samples_flac_b = read_metrics_from_flac_by_bloc(filename_flac);
    println!("{:?}", samples_flac);
    println!("{:?}", samples_flac_b);
    assert_eq!(samples_flac_b, samples_flac);
    //let samples_flac_in_interval = read_metrics_from_flac_in_interval(filename_flac, 5, 7);
    println!("Sample Flac {:?}", samples_flac.len());
    println!("Sample Flac {:?}", samples_flac_b.len());
}
