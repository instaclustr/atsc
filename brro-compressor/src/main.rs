use log::{info, debug};
use clap::{Parser, command, arg};

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
    // The baseline work should be: streaming read -> compress -> streaming write
    // 1st Stage: Create a streaming reader
    // 2nd Stage: Create a streaming writer
    // The system should be able to read a stream and write it within the defined architecture
    
}