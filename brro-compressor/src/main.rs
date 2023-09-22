use log::{info, debug};
use clap::{Parser, command, arg};

#[macro_use]
extern crate log;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: String,

    /// Set this file to process a whole directory
    #[arg(short, action)]
    directory: bool,

}

fn main() {
    env_logger::init();
    let arguments = Args::parse();
    debug!("{:?}", arguments);
    info!("Start!");
}