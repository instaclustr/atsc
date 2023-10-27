fn create_wavbrro_header(channels: Option<i32>, bitdepth: u16, samplerate: u32) -> WavSpec { 
    hound::WavSpec {
        channels: channels.unwrap_or(4) as u16,
        sample_rate: samplerate,
        bits_per_sample: bitdepth,
        sample_format: hound::SampleFormat::Int
    }
}