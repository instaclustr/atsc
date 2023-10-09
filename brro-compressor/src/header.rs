use bincode::{Decode, Encode};

/// This will write the file headers
#[derive(Encode, Decode, Debug, Clone)]
pub struct CompressorHeader {
    initial_segment: [u8; 4],
    // We should go unsigned
    frame_count: i16,
}

impl CompressorHeader{
    pub fn new() -> Self {
        CompressorHeader{ 
            initial_segment: *b"BRRO",
            // We have to limit the bytes of the header
            frame_count: 0
        }
    }

    pub fn add_frame (&mut self) {
        self.frame_count += 1;
    } 
}