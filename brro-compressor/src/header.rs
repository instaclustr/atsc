/// This will write the file headers

pub struct CompressorHeader {
    initial_segment: u8,
    segment_count: i16,
}

impl CompressorHeader{
    pub fn new() -> Self {
        CompressorHeader{ initial_segment: 0, segment_count: 0}
    }

    pub fn inc_segment_count (&mut self) {
        self.segment_count += 1;
    } 
}