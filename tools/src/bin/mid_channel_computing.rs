use clap::Parser;
use hound::{WavSpec, WavWriter};

fn read_metrics_from_wav(filename: &str) -> Vec<f64> {
    let r_reader = hound::WavReader::open(filename);
    let mut reader = match r_reader {
        Ok(reader) => reader,
        Err(_err) => {
            return Vec::new();
        }
    };
    let num_channels = reader.spec().channels as usize;
    let bit_depth = reader.spec().bits_per_sample;

    let mut raw_data: Vec<f64> = Vec::new();

    // This is a very special case where the wav file holds a 64bit float spread over 4 16bit channels
    if num_channels == 4 && bit_depth == 16 {
        // Iterate over the samples and channels and push each sample to the vector
        let mut u64_holder: [u16; 4] = [0, 0, 0, 0];
        let mut current_channel: usize = 0;

        for sample in reader.samples::<i16>() {
            u64_holder[current_channel] = sample.unwrap() as u16;
            current_channel += 1;
            if current_channel == num_channels {
                raw_data.push(join_u16_into_f64(u64_holder));
                current_channel = 0;
            }
        }
    } else {
        for sample in reader.samples::<i16>() {
            raw_data.push(sample.unwrap() as f64);
        }
    }

    raw_data
}

fn join_u16_into_f64(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64)
        | ((bits[1] as u64) << 16)
        | ((bits[2] as u64) << 32)
        | ((bits[3] as u64) << 48);

    f64::from_bits(u64_bits)
}

fn write_optimal_int_wav(filename: &str, data: Vec<i16>, bitdepth: i32, channels: i32) {
    let header: WavSpec = generate_wav_header(Some(channels), bitdepth as u16, 8000);
    let file_path = format!("{filename}.wav");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(file_path)
        .unwrap();
    let mut wav_writer = WavWriter::new(file, header).unwrap();
    for sample in data {
        let _ = wav_writer.write_sample(sample as i8);
    }
    let _ = wav_writer.finalize();
}

fn generate_wav_header(channels: Option<i32>, bitdepth: u16, samplerate: u32) -> WavSpec {
    hound::WavSpec {
        channels: channels.unwrap_or(4) as u16,
        sample_rate: samplerate,
        bits_per_sample: bitdepth,
        sample_format: hound::SampleFormat::Int,
    }
}

fn calculate_mid_channel(left: Vec<f64>, right: Vec<f64>) -> (Vec<i16>, Vec<i16>) {
    // We might have different sizes
    let min_size = left.len().min(right.len());
    let mut mid: Vec<i16> = Vec::with_capacity(min_size);
    let mut sides: Vec<i16> = Vec::with_capacity(min_size);
    // Formulas are easy, mid = 0.5*(left + right), sides = 0.5 * (left – right)
    for i in 0..min_size {
        mid.push((left[i] as i16 + right[i] as i16) / 2);
        sides.push((left[i] as i16 - right[i] as i16) / 2);
    }
    (mid, sides)
}

#[derive(Parser, Default, Debug)]
struct Arguments {
    /// First wav file
    file_one: String,
    /// Second wav file
    file_two: String,
    /// reverse the process, picks the first file, writes in the second
    #[arg(long, action)]
    reverse: bool,
    /// Don't write a file, just dump the calculated contents
    #[arg(long, action)]
    dump: bool,
}

fn main() {
    let args = Arguments::parse();
    println!("{:?}", args);

    let binding_a: Vec<f64> = read_metrics_from_wav(&args.file_one);
    let binding_b: Vec<f64> = read_metrics_from_wav(&args.file_two);
    let (mid, sides) = calculate_mid_channel(binding_a, binding_b);
    if args.dump {
        println!("Mid: {:?}", mid);
        println!("Sides: {:?}", sides);
    } else {
        write_optimal_int_wav("mid", mid, 16, 1);
        write_optimal_int_wav("side", sides, 8, 1);
    }
}
