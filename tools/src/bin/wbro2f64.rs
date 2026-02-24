use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
use wavbrro::wavbrro::WavBrro;

#[derive(Debug, Parser)]
#[command(author, version, about = "Convert a .wbro file to raw little-endian f64 bytes")]
struct Args {
    /// Input `.wbro` file.
    input: PathBuf,
    /// Output `.f64` file (raw little-endian f64 bytes).
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let samples = WavBrro::from_file(&args.input)?;
    let mut out = std::fs::File::create(&args.output)?;
    for v in samples {
        out.write_all(&v.to_le_bytes())?;
    }
    Ok(())
}

