use std::{env::args, io::{self, BufRead}};
use hound::{WavSpec, WavWriter};
use clap::{Parser, command, arg};
use regex::Regex;
use median::Filter;

#[derive(Debug)]
enum MetricTag {
    Percent(i32), // If it is a percent reduce significant digits to 2
    Duration(i32), // if it is a duration reduce precision to 1 microsecond
    NotFloat, // A metric that has a float representation but shouldn't (Eg. Precision is not needed)
    QuasiRandom, // A metric that exhibits a quasi random sample behavior. (E.g. Network deltas, heap memory)
    Bytes(i32), // Data that is in bytes... Make it MB, or KB
    Other // Everything else
}

impl MetricTag {
    fn from_float(&self, x: f64) -> i64 {
        match self {
            MetricTag::Other => {
                0
            }
            MetricTag::NotFloat | MetricTag::QuasiRandom => {
                x as i64
            }
            MetricTag::Percent(y) => {
                to_multiply_and_truncate(x, *y)
            }
            MetricTag::Duration(y) => {
                to_multiply_and_truncate(x, *y)
            },
            MetricTag::Bytes(y) => {
                (x as i64)/(*y as i64)
            }
        }
    }
}

/*
Reads a WAV file, checks the channels and the information contained there. From that
information takes a decision on the best channel, block size and bitrate for the BRRO
encoders.
*/

/* Read a WAV file,  */
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

fn generate_wav_header(channels: Option<i32>, bitdepth: u16, samplerate: u32) -> WavSpec {
    let spec = hound::WavSpec {
        channels: channels.unwrap_or(4) as u16,
        // TODO: Sample rate adaptations
        sample_rate: samplerate,
        bits_per_sample: bitdepth,
        sample_format: hound::SampleFormat::Int
    };
    return spec;
}

/// Write a WAV file with the outputs of data analysis for float data
fn write_optimal_wav(filename: &str, data: Vec<f64>, bitdepth: i32, dc: i64, channels: i32) {
    // Make DC a float for operations
    let fdc = dc as f64;
    let header: WavSpec = generate_wav_header(Some(channels), bitdepth as u16, 8000);
    let mut file_path = format!("{}", filename);
    file_path.truncate(file_path.len() - 4);
    file_path = format!("{}_OPT.wav", file_path);
    let file = std::fs::OpenOptions::new().write(true).create(true).read(true).open(file_path).unwrap();
    let mut wav_writer = WavWriter::new(file, header).unwrap();
    for sample in data {
        let _ = match bitdepth {
            8 =>  wav_writer.write_sample(as_i8(sample-fdc)),
            16 => wav_writer.write_sample(as_i16(sample-fdc)),
            _ => wav_writer.write_sample(as_i32(sample-fdc))
        };
    }
}

fn write_optimal_int_wav(filename: &str, data: Vec<i64>, bitdepth: i32, dc: i64, channels: i32) {
    let header: WavSpec = generate_wav_header(Some(channels), bitdepth as u16, 8000);
    let mut file_path = format!("{}", filename);
    file_path.truncate(file_path.len() - 4);
    file_path = format!("{}_OPT.wav", file_path);
    let file = std::fs::OpenOptions::new().write(true).create(true).read(true).open(file_path).unwrap();
    let mut wav_writer = WavWriter::new(file, header).unwrap();
    for sample in data {
        let _ = match bitdepth {
            8 =>  wav_writer.write_sample((sample-dc) as i8),
            16 => wav_writer.write_sample((sample-dc) as i16),
            _ => wav_writer.write_sample((sample-dc) as i32)
        };
    }
}

fn as_i8(value: f64) -> i8 {
    return split_n(value).0 as i8;
}

fn as_i16(value: f64) -> i16 {
    return split_n(value).0 as i16;
}

fn as_i32(value: f64) -> i32 {
    return split_n(value).0 as i32;
}

// Split a float into an integer
fn split_n(x: f64) -> (i64, f64) {
    const FRACT_SCALE: f64 = 1.0 / (65536.0 * 65536.0 * 65536.0 * 65536.0); // 1_f64.exp(-64)
    const STORED_MANTISSA_DIGITS: u32 = f64::MANTISSA_DIGITS - 1;
    const STORED_MANTISSA_MASK: u64 = (1 << STORED_MANTISSA_DIGITS) - 1;
    const MANTISSA_MSB: u64 = 1 << STORED_MANTISSA_DIGITS;

    const EXPONENT_BITS: u32 = 64 - 1 - STORED_MANTISSA_DIGITS;
    const EXPONENT_MASK: u32 = (1 << EXPONENT_BITS) - 1;
    const EXPONENT_BIAS: i32 = (1 << (EXPONENT_BITS - 1)) - 1;

    let bits = x.to_bits();
    let is_negative = (bits as i64) < 0;
    let exponent = ((bits >> STORED_MANTISSA_DIGITS) as u32 & EXPONENT_MASK) as i32;

    let mantissa = (bits & STORED_MANTISSA_MASK) | MANTISSA_MSB;
    let mantissa = if is_negative { -(mantissa as i64) } else { mantissa as i64 };

    let shl = exponent + (64 - f64::MANTISSA_DIGITS as i32 - EXPONENT_BIAS + 1);
    if shl <= 0 {
        let shr = -shl;
        if shr < 64 { // x >> 0..64
            let fraction = ((mantissa as u64) >> shr) as f64 * FRACT_SCALE;
            (0, fraction)
        } else { // x >> 64..
            (0, 0.0)
        }
    } else {
        if shl < 64 { // x << 1..64
            let int = mantissa >> (64 - shl);
            let fraction = ((mantissa as u64) << shl) as f64 * FRACT_SCALE;
            (int, fraction)
        } else if shl < 128 { // x << 64..128
            let int = mantissa << (shl - 64);
            (int, 0.0)
        } else { // x << 128..
            (0, 0.0)
        }
    }
}

fn join_u16_into_f64(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64) |
                ((bits[1] as u64) << 16) |
                ((bits[2] as u64) << 32) |
                ((bits[3] as u64) << 48);
    
    let f64_value = f64::from_bits(u64_bits);
    f64_value
}

fn get_max(a: i32, b: i32) -> i32 {
    a.max(b)
}

/// Converts a float via multiplication and truncation 
fn to_multiply_and_truncate(number: f64, mul: i32) -> i64 {
    (number*mul as f64) as i64
}

fn to_median_filter(data: &Vec<f64>) -> Vec<i64> {
    let mut filtered = Vec::with_capacity(data.len());
    // 10minutes of data
    let mut filter = Filter::new(50);
    for point in data {
        let point_int = MetricTag::QuasiRandom.from_float(*point);
        let median = filter.consume(point_int);
        filtered.push(median)
    }
    filtered
}

/// Check the type of metric and tag it
fn tag_metric(filename: &str) -> MetricTag {
    /// Should sort this by the probability of each tag, so the ones that are more common are dealt first
    // If it says percent_ or _utilization
    let mut regex = Regex::new(r"(?m)percent_|_utilization").unwrap();
    if regex.captures(filename).is_some() {
        // 2 significant digits resolution (Linux resolution)
        return MetricTag::Percent(100);
    }
    // if it says _client_request
    regex = Regex::new(r"(?m)_client_request").unwrap();
    if regex.captures(filename).is_some() {
        // Fractional requests are nothing but an averaging artifact
        return MetricTag::NotFloat;
    }
    // if it says _seconds
    regex = Regex::new(r"(?m)_seconds").unwrap();
    if regex.captures(filename).is_some() {
        // 1 micro second resolution
        return MetricTag::Duration(1_000_000);
    }
    // if it says _seconds
    regex = Regex::new(r"(?m)_networkindelta|_networkoutdelta|_heapmemoryused_").unwrap();
    if regex.captures(filename).is_some() {
        return MetricTag::QuasiRandom;
    }
    MetricTag::Other
}

/// Go through the data, check min and max values, DC Component
/// Check if data fits in 8,16,24,32 bits. If so reduce it to a single channel with
/// those bit depths.
fn analyze_data(data: &Vec<f64>) -> (i32, i64, bool) {
    let mut min: f64 = 0.0;
    let mut max: f64 = 0.0;
    let mut fractional = false;
    for value in data {
        let t_value = *value;
        if split_n(t_value).1 != 0.0 { fractional = true;}
        if t_value > max { max = t_value};
        if t_value < min { min = t_value};
    }
    // Check max size of values
    // For very large numbers (i32 and i64), it might be ideal to detect the dc component
    // of the signal. And then remove it later
    let max_int = split_n(max).0; // This is the DC component
    let min_int = split_n(min).0;

    // If fractional is it relevant?
    let max_frac = split_n(max).1;

    // Finding the bitdepth without the DC component
    let recommended_bitdepth = find_bitdepth(max_int-min_int, min_int);
    if !fractional {
        println!(" Recommended Bitdepth: {} ", recommended_bitdepth);
    } else {
        println!(" Fractional, Recommended Bitdepth: {}, Fractions max: {}", recommended_bitdepth, max_frac);
    }
    (recommended_bitdepth, min_int, fractional)
}

fn analyze_int_data(data: &Vec<i64>) -> (i32, i64) {
    let mut min: i64 = 0;
    let mut max: i64 = 0;
    for value in data {
        let t_value = *value;
        if t_value > max { max = t_value};
        if t_value < min { min = t_value};
    }

    let recommended_bitdepth = find_bitdepth(max-min, min);
    println!(" Recommended Bitdepth: {} ", recommended_bitdepth);
    (recommended_bitdepth, min)
}

fn find_bitdepth(max_int: i64, min_int: i64) -> i32 {
    // Check where those ints fall into
    let bitdepth = match max_int {
        _ if max_int <= u8::MAX.into() => 8,
        _ if max_int <= i16::MAX.into() => 16,
        _ if max_int <= i32::MAX.into() => 32,
        _ => 64
    };

    let bitdepth_signed = match min_int {
        _ if min_int == 0 => 8,
        _ if min_int >= i16::MIN.into() => 16,
        _ if min_int >= i32::MIN.into() => 32,
        _ => 64
    };

    let recommended_bitdepth = get_max(bitdepth, bitdepth_signed);
    recommended_bitdepth
}

fn process_args(filename: &str, optimize: bool) {
    print!("\nFile: {} ,", filename);
    let mut bitdepth = 64;
    let mut dc_component: i64 = 0;
    let mut fractional = true;
    let wav_data = read_metrics_from_wav(filename);
    println!("Original Data: {:?}", wav_data);
    // Depending on Metric Tag, apply a transformation
    let tag = tag_metric(filename);
    println!("Tag: {:?}", tag);
    let iwav_data =  match tag {
        MetricTag::Other => Vec::new(),
        MetricTag::QuasiRandom => to_median_filter(&wav_data),
        _ => { wav_data
            .iter()
            .map(|x| tag.from_float(*x))
            .collect()
            }      
    };
    // We split the code here
    if iwav_data.len() > 0 {
        fractional = false;
        println!("int Data: {:?}", iwav_data);
        (bitdepth, dc_component) = analyze_int_data(&iwav_data);
    } else {
        (bitdepth, dc_component, fractional) = analyze_data(&wav_data);
    }
    if bitdepth == 64 || fractional { 
        //println!("No optimization, exiting");
        std::process::exit(0); 
    } else if optimize {
        println!("\nWriting optimal file!");
        match iwav_data.len() {
            0 => write_optimal_wav(filename, wav_data, bitdepth, dc_component, 1),
            _ => write_optimal_int_wav(filename, iwav_data, bitdepth, dc_component, 1)
        }
    }
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    #[arg(short, long)]
    input: String,

    /// Write a new file with optimized settings
    #[arg(short)]
    write: bool,

    /// Samplerate to generate the optimized file
    #[arg(short, long, default_value_t = 8000)]
    samplerate: u32,

    // Write samples to a file
    #[arg(short)]
    save_raw: bool,
}

fn main() {
    // How to break the float part??? --> THERE ARE NO FLOATS!
    // https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/6/html/deployment_guide/s2-proc-stat
    let arguments: Vec<String> = args().collect();
    let mut writeout: bool = false;
    match arguments.len() {
        1 => {},
        2 => { if arguments[1].as_str() == "-w" {
                        writeout=true; 
                } else {
                    process_args(arguments[1].as_str(), writeout);
                    std::process::exit(0); 
                }
            },
        3 => { if arguments[1].as_str() == "-w" {
                    process_args(arguments[2].as_str(), true);
                    std::process::exit(0); 
                } else {
                    println!("Wrong arguments!");
                    std::process::exit(1); 
                }
            },
        _  => { println!("Too many arguments!");
               std::process::exit(1); 
            }
    }
        
    // Read STDIN
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("Could not read line from standard in");
        process_args(line.trim(), writeout);
    }
}