/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone)]
pub struct CompressorHeader {
    initial_segment: [u8; 4],
    // We should go unsigned
    frame_count: i16,
}

impl CompressorHeader {
    pub fn new() -> Self {
        CompressorHeader {
            initial_segment: *b"BRRO",
            // We have to limit the bytes of the header
            frame_count: 0,
        }
    }

    pub fn add_frame(&mut self) {
        self.frame_count += 1;
    }
}
