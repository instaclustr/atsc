/// This will write the file headers

pub struct CompressorHeader {
    initial_segment: [u8; 4],
    frame_count: i16,
}

impl CompressorHeader{
    pub fn new() -> Self {
        CompressorHeader{ 
            initial_segment: *b"BRRO",
            frame_count: 0
        }
    }

    pub fn add_frame (&mut self) {
        self.frame_count += 1;
    } 
}