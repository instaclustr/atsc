use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect an ATSC v2 stream.
    Inspect { input: std::path::PathBuf },
}

fn main() {
    env_logger::init();
    let _args = Args::parse();
    // CLI will be implemented after the library API (PLAN.md Phase 4).
}
