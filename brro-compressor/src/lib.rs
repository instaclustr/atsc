#![allow(clippy::new_without_default)]

// Lucas - Once the project is far enough along I strongly reccomend reenabling dead code checks
#![allow(dead_code)]

pub mod compressor;
pub mod frame;
pub mod preprocessor;
pub mod utils;
pub mod header;
pub mod data;
pub mod compare;

pub mod optimizer;
pub mod types;
