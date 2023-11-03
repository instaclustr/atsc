use wavbrro::wavbrro::WavBrro;
use log::debug;
//use clap::{arg, command, Parser};

fn main() {
    env_logger::init();
    //let arguments = Args::parse();
    //debug!("{:?}", arguments);
    debug!("Test, 1,23");
    let mut wb = WavBrro::new();
    wb.add_sample(34.1);
}