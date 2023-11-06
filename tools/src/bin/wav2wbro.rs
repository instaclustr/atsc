use std::io::Read; 
use std::path::{Path, PathBuf};
use std::fs::File;
use wavbrro::wavbrro::WavBrro;
use log::debug;
use clap::{arg, command, Parser};


// --- Legacy stuff to read brro "wav" files ---
fn is_wav_file(file_path: &Path) -> bool {
    // Open the file for reading and read the first 12 bytes (header) of the file
    let mut file = File::open(file_path).expect("Can't open file!");
    let mut header = [0u8; 12];
    file.read_exact(&mut header).expect("File too small!");

    // Check if the file starts with "RIFF" and ends with "WAVE" in the header
    header.starts_with(b"RIFF") && &header[8..12] == b"WAVE"
}

fn read_metrics_from_wav(filename: &str) -> Vec<f64> {
    let r_reader = hound::WavReader::open(filename);
    let mut reader = match r_reader {
        Ok(reader) => reader,
        Err(_err) => { return Vec::new(); }
    };
    let num_channels = reader.spec().channels as usize;

    let mut raw_data: Vec<f64> = Vec::new();
    let mut u64_holder: [u16; 4] = [0, 0, 0, 0];

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
    raw_data
}

fn join_u16_into_f64(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64) |
        ((bits[1] as u64) << 16) |
        ((bits[2] as u64) << 32) |
        ((bits[3] as u64) << 48);


    f64::from_bits(u64_bits)
}
// --- Legacy ends (I need to stop lying to myself...) ---

#[derive(Parser, Default, Debug)]
#[command(author, version, about="WAV to WAVBRRO converter", long_about = None)]
struct Args {
    /// input file
    input: PathBuf,

    /// Verbose output, dumps everysample in the input file (for compression) and in the ouput file (for decompression)
    #[arg(long, action)]
    validate: bool,
}

fn main() {
    env_logger::init();
    let arguments = Args::parse();
    debug!("{:?}", arguments);
    let filename = arguments.input.to_str().unwrap();
    assert!(is_wav_file(&arguments.input));
    let wav_data = read_metrics_from_wav(filename);
    let mut wb = WavBrro::new();
    wav_data.iter().for_each(|x| wb.add_sample(*x));
    // Write the file
    let wavbrro_file = format!("{}wbro", filename.strip_suffix("wav").unwrap()); 
    wb.to_file(Path::new(&wavbrro_file));
    // Checking the results
    if arguments.validate {
        let brro_data = wb.get_samples();
        assert_eq!(wav_data, brro_data);
    }
}