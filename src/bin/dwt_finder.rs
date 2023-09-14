use dtw_rs::{ParameterizedAlgorithm, DynamicTimeWarping, Algorithm};
use clap::Parser;

fn read_metrics_from_wav(filename: &str) -> Vec<f64> {
    let r_reader = hound::WavReader::open(filename);
    let mut reader = match r_reader {
        Ok(reader) => reader,
        Err(_err) => { return Vec::new();}
    };
    let num_channels = reader.spec().channels as usize;

    let mut raw_data: Vec<f64> = Vec::new();
    let mut u64_holder: [u16; 4] = [0,0,0,0]; 

    // Iterate over the samples and channels and push each sample to the vector
    let mut current_channel: usize = 0;
    for sample in reader.samples::<i16>() {
        u64_holder[current_channel] = sample.unwrap() as u16;
        current_channel += 1;
        if current_channel == num_channels {
            raw_data.push(join_u16_into_f64(u64_holder));
            current_channel = 0;
        }
    }
    return raw_data;
}

fn join_u16_into_f64(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64) |
                ((bits[1] as u64) << 16) |
                ((bits[2] as u64) << 32) |
                ((bits[3] as u64) << 48);

    let f64_value = f64::from_bits(u64_bits);
    f64_value
}

#[derive(Parser,Default,Debug)]
struct Arguments {
    /// First wav file
    file_one: String,
    /// Second wav file
    file_two: String,
    /// Distance
    distance: usize,
    /// Block size
    block: usize
}

fn main() {
    let args = Arguments::parse();
    println!("{:?}", args);

    let binding_a: Vec<f64> = read_metrics_from_wav(&args.file_one);
    let binding_b: Vec<f64> = read_metrics_from_wav(&args.file_two);
    let vec_slices_a: Vec<&[f64]> = binding_a.chunks(args.block).collect();
    let vec_slices_b: Vec<&[f64]> = binding_b.chunks(args.block).collect();
    let data_a = vec_slices_a[0];
    let data_b = vec_slices_b[0];

    let param = dtw_rs::Restriction::Band(args.distance);
    let dtw = DynamicTimeWarping::with_param(data_a, data_b, param);

    println!("Path: {:?}, Distance: {}", dtw.path(), dtw.distance());
}