use std::fs::File;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::errors::Error;
use symphonia::core::codecs::{DecoderOptions};
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};
use symphonia::core::io::MediaSourceStream;

fn extract_flac_content_from_interval(start_time: u64, end_time: u64)-> Result<Vec<i16>, Error> {
    let file_path = "2023-05-11_15-11-19.flac";

    let file = Box::new(File::open(file_path).unwrap());
    let reader = MediaSourceStream::new(file, Default::default());

    let format_options = FormatOptions::default();
    let decoder_options = DecoderOptions::default();
    let metadata_opts: MetadataOptions = Default::default();

    // Lets probe
    let probed = symphonia::default::get_probe().format(Hint::new().mime_type("FLaC"), reader, &format_options, &metadata_opts).unwrap();
    let mut format_reader = probed.format;
    let track = format_reader.default_track().unwrap();
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &decoder_options).unwrap();

    let sample_rate = format_reader.tracks()[0].codec_params.sample_rate.unwrap();

    let seek_point = SeekTo::Time {
        time: Time::new(start_time, 0.0),
        track_id: Some(format_reader.tracks()[0].id) };

    let end_point_ts = TimeBase::new(1, sample_rate).calc_timestamp(Time::new(end_time, 0.0));
    
    // Prepare to store data, with Optimal Seek (less performance) this can be a static value, otherwise will stay like this
    let mut buffer = Vec::new();
    let mut sample_buf = None;
    // Seek to the correct point
    let initial_point = format_reader.seek(SeekMode::Coarse, seek_point);
    match initial_point {
        Ok(point) => { println!("Initial point: {:?}", point);},
        Err(err) => { panic!("Unable to find starting point! Error: {}", err); }
    }
    
    // Not stopping on the required time (yet)
    loop {
        // Get the next packet from the media format.
        let packet = match format_reader.next_packet() {
            Ok(packet) => packet,
            Err(err) => {
                // A unrecoverable error occured, halt decoding.
                panic!("{}", err);
            }
        };
        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Consume the decoded audio samples (see below).
                if sample_buf.is_none() {
                    // Get the audio buffer specification.
                    let spec = *decoded.spec();
                    // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                    let duration = decoded.capacity() as u64;
                    // Create the f32 sample buffer.
                    sample_buf = Some(SampleBuffer::<i16>::new(duration, spec));
                }
                if let Some(buf) = &mut sample_buf {
                    //buf.copy_interleaved_ref(decoded);
                    buf.copy_planar_ref(decoded);
                    for sample in buf.samples() {
                        buffer.push(*sample);
                    }
                    //print!("\rSamples decoded: {:?} samples", buffer);
                }
            }
            Err(Error::IoError(_)) => {
                // The packet failed to decode due to an IO error, skip the packet.
                continue;
            }
            Err(Error::DecodeError(_)) => {
                // The packet failed to decode due to invalid data, skip the packet.
                continue;
            }
            Err(err) => {
                // An unrecoverable error occured, halt decoding.
                panic!("{}", err);
            }
        }
        if packet.ts >= end_point_ts {
            // Stop the loop, we are done!
            println!("Packet TS : {:?}, Packet Time: {:?}", packet.ts, end_point_ts);
            break;
        }
    }
    Ok(buffer)
}


fn main() {
    let start_time = 5; // Start time in seconds
    let end_time = 7; // End time in seconds

    match extract_flac_content_from_interval(start_time, end_time) {
        Ok(buffer) => println!("Sample Len: {}", buffer.len()),
        Err(err) => println!("{:?}", err),
    }
}
