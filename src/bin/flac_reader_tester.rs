/*
This file compares a FLAC and WAV file and if the content is identical
Also good to test if the FLAC and WAV read routines are good
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
        //println!("{:?}", sample.unwrap());
        //break;
        samples.push(sample.unwrap() as i16);
    }
    return samples;
}

fn read_metrics_from_flac_in_interval(filename: &str, start: u32, end: u32) -> Vec<i16> {
    let mut reader = claxon::FlacReader::open(filename).unwrap();
    // Create a vector to hold the audio data
    let start_sample = start * reader.streaminfo().sample_rate;
    let end_sample = end * reader.streaminfo().sample_rate;
    //let mut samples = Vec::with_capacity(reader.streaminfo().samples.unwrap() as usize);
    let mut samples: Vec<i16> = Vec::new();
    let mut i = 0;
    for sample in reader.samples() {
        if  start_sample <=i && i <= end_sample {
            samples.push(sample.unwrap() as i16);
        }
        else if i > end_sample {
            break;
        }
        i+=1;
    }
    return samples;
}

fn main() {
    println!("Testing, does FLAC reading is the same as WAV?");
    let filename = "2023-05-11_15-11-19.wav";
    let filename_flac = "Nums_5dot1_24_48000.flac";
    let filename_flac_single = "3_single_channel.flac";
    //let samples = read_metrics_from_wav(filename);
    //println!("{:?}", samples);
    let samples_flac = read_metrics_from_flac(filename_flac);
    let samples_flac_b = read_metrics_from_flac(filename_flac_single);
    //println!("{:?}", samples_flac);
    //assert_eq!(samples, samples_flac);
    //let samples_flac_in_interval = read_metrics_from_flac_in_interval(filename_flac, 5, 7);
    println!("Sample Flac {:?}", samples_flac.len());
    println!("Sample Flac {:?}", samples_flac_b.len());
    
}