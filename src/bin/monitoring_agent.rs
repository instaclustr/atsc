use sysinfo::{CpuExt, System, SystemExt};
use std::fs::OpenOptions;
use hound::WavWriter;
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

fn append_to_fat_wav_file(file_path: &str, data: Vec<i32>) -> Result<(), hound::Error> {
    let file = OpenOptions::new().write(true).read(true).open(file_path).unwrap();
    let mut writer = WavWriter::new_append(file)?;
        for value in data {
            writer.write_sample(value).unwrap();
        }
    writer.finalize().unwrap();
    Ok(())
}

fn append_to_stereo_wav_file(file_path: &str, data: Vec<(i16, i16)>) -> Result<(), hound::Error> {
    // We should check if the file is stereo...
    let file = OpenOptions::new().write(true).read(true).open(file_path).unwrap();
    let mut writer = WavWriter::new_append(file)?;
    // Stereo samples are interleaved
    for value in data {
            writer.write_sample(value.0).unwrap();
            writer.write_sample(value.1).unwrap();
    }
    writer.finalize().unwrap();
    Ok(())
}

fn process_percentage(value: f32) -> i16 {
    return (value * 100.0).round() as i16;
}

fn kb_to_mb(value: u64) -> i16 {
    // Can only hold up to 32GB here, probably split this into 2 channels?
    return (value / 1024) as i16;
}

fn bytes_to_kb(value: u64) -> i32 {
    // Can only hold up to 32GB here, probably split this into 2 channels?
    return (value / 1024) as i32;
}

fn split_into_channels(value: i64) -> (i32, i32) {
    return((value >> 32) as i32, value as i32)
}

fn main() {
    
    let mut sys = System::new_all();
    // WAV Spec
    // Set the correct WAV file header (16bit PCM, 8kHZ)
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int
    };
    // Memory has large numbers, we can go with 32Bit per sample (16bit PCM, 8kHZ)
    let fat_spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Int
    };

    let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");

    // Need one file for each CPU
    for cpu in sys.cpus() {
        let file_name = format!("cpu_{}_{}.wav", cpu.name(),ts);
        let file_path = format!("./{}", file_name);
        let file = OpenOptions::new().write(true).create(true).read(true).open(file_path.clone());
        // Create a new WAV file
        let writer = WavWriter::new(file.unwrap(), spec);
    }
    let mem_file_name = format!("memory_{}.wav", ts);
    let mem_file_path = format!("./{}", mem_file_name);
    let file = OpenOptions::new().write(true).create(true).read(true).open(mem_file_path.clone());
    // Create a new WAV file
    let writer = WavWriter::new(file.unwrap(), fat_spec);

    loop {
        sys.refresh_cpu(); // Refreshing CPU information.
        sys.refresh_memory(); // Refreshing memory information.
        for cpu in sys.cpus() {
            let cpu_data_vec: Vec<i16> = vec![process_percentage(cpu.cpu_usage())];
            let file_name = format!("cpu_{}_{}.wav", cpu.name(),ts);
            let file_path = format!("./{}", file_name);
            append_to_wav_file(file_path.as_str(), cpu_data_vec);
        
        }
        // Now for memory
        let available_mem = sys.available_memory();
        // This will break if somehow the system has more than 2TB of available memory
        append_to_fat_wav_file(mem_file_path.as_str(), vec![bytes_to_kb(available_mem)]);
        // Sleeping for 500 ms to let time for the system to run for long
        // enough to have useful information.
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
