use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use bincode::{Decode, Encode};
use crate::frame::CompressorFrame;


#[derive(Encode, Decode, Debug, Clone)]
pub struct FramesManager {
    frames: Vec<Arc<CompressorFrame>>,
    frame_pointers: HashMap<u64, Arc<CompressorFrame>>,
}

impl FramesManager {
    pub fn new() -> Self {
        FramesManager {
            frames: Vec::new(),
            frame_pointers: HashMap::new(),
        }
    }

    // Method to add a frame, implementing the ghost frame logic
    pub fn add_frame(&mut self, frame: CompressorFrame) {

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        frame.hash(&mut hasher);
        let frame_hash = hasher.finish();

        match self.frame_pointers.entry(frame_hash) {
            Entry::Occupied(entry) => {
                // If a frame with the same hash already exists, reuse the existing Arc
                let existing_arc = entry.get().clone();
                self.frames.push(existing_arc);
            },
            Entry::Vacant(entry) => {
                // If it's a new frame, create a new Arc, insert it into the map and the list
                let frame_arc = Arc::new(frame);
                entry.insert(frame_arc.clone());
                self.frames.push(frame_arc);
            },
        }
    }
    // Returns a new Vec containing all frame references (as Arcs).
    pub fn get_all_frames(&self) -> Vec<Arc<CompressorFrame>> {
        self.frames.clone()
    }
}