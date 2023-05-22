use sysinfo::{CpuExt, System, SystemExt};
use std::fs::{OpenOptions, File};
use std::io::BufWriter;
use hound::{WavReader, WavWriter};
use chrono;

fn append_to_wav_file(file_path: &str, data: Vec<i16>) -> Result<(), hound::Error> {
    let file = OpenOptions::new().write(true).read(true).open(file_path).unwrap();
    let mut writer = WavWriter::new_append(file)?;
        for value in data {
            writer.write_sample(value).unwrap();
        }
    writer.finalize().unwrap();
    Ok(())
}

fn process_percentage(value: f32) -> i16 {
    return (value * 100.0).round() as i16;
}

fn main() {
    let mut sys = System::new_all();
    let file_name = format!("{}.wav", chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let file_path = format!("./{}", file_name);
    let file = OpenOptions::new().write(true).create(true).read(true).open(file_path.clone());
    // Set the correct WAV file header (16bit PCM, 8kHZ)
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };
    // Create a new WAV file
    let writer = WavWriter::new(file.unwrap(), spec);

    loop {
        sys.refresh_cpu(); // Refreshing CPU information.
        let cpu_data_vec: Vec<i16> = sys.cpus().iter().map(|cpu| process_percentage(cpu.cpu_usage())).collect();
        append_to_wav_file(file_path.as_str(), cpu_data_vec);
        // Send information through a TCP socket.


        // Sleeping for 500 ms to let time for the system to run for long
        // enough to have useful information.
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
