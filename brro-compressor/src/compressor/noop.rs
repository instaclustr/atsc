use log::{info, debug};

pub struct Noop {
    pub name: String,
}

impl Noop {
    pub fn new() -> Result<Self, String>{
        debug!("Noop!");
        Ok(Noop { name: "Test".to_string() })
    }
}