use std::fs::{OpenOptions, File};
use std::io::BufWriter;
use hound::{WavReader, WavWriter};
use chrono;

fn append_to_wav_file(file_path: &str, data: &[i16]) -> Result<(), hound::Error> {
    let file = OpenOptions::new().write(true).read(true).open(file_path).unwrap();
    let mut writer = WavWriter::new_append(file)?;
    // Write the new data to the end of the file
        for value in data {
            writer.write_sample(*value).unwrap();
        }
    writer.finalize().unwrap();
    Ok(())
}

fn main() {
    // Create a  WAV file with current timedate as name
    let file_name = format!("{}.wav", chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let file_path = format!("./{}", file_name);
    let mut file = OpenOptions::new().write(true).create(true).read(true).open(file_path);
    // Set the correct WAV file header (16bit PCM, 8kHZ)
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };
    // Create a new WAV file
    let writer = WavWriter::new(file.unwrap(), spec);

    // Read from a TCP socket and write to the WAV file
    

}
