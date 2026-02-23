# Agents Guide (fft-compression / ATSC workspace)

This repository is a Rust workspace containing ATSC (Advanced Time Series Compressor) and supporting crates. This file provides **project-specific guidance and guardrails** for AI-assisted changes, especially around on-disk formats and performance-sensitive code.

## Codebase map (high-signal entry points)

- **CLI**: `atsc/src/main.rs`
  - Reads `.wbro` or `.csv`, produces `.bro` (values) and `.vsri` (timestamps when CSV).
- **Stream container (on-disk payload)**: `atsc/src/data.rs` (`CompressedStream`)
- **Frame container (per chunk)**: `atsc/src/frame/mod.rs` (`CompressorFrame`)
- **Header / magic**: `atsc/src/header.rs` (`"BRRO"`)
- **Compressors**: `atsc/src/compressor/`
  - FFT: `atsc/src/compressor/fft.rs`
  - Polynomial / IDW: `atsc/src/compressor/polynomial.rs`
  - Constant: `atsc/src/compressor/constant.rs`
  - VSRI (timestamps): `vsri/src/lib.rs` (separate workspace crate)
- **Chunking / planning**: `atsc/src/optimizer/mod.rs` (`OptimizerPlan`)

## How to build, test, lint, benchmark

- **Build**: `cargo build --release`
- **Format**: `cargo fmt`
- **Clippy**: `cargo clippy --workspace --all-targets`
- **Unit tests**: `cargo test --workspace`
- **Benchmarks (Criterion)**:
  - `cargo bench -p atsc --bench fft_bench`
  - `cargo bench -p atsc --bench polynomial_bench`

If you change compressor logic, prefer adding/adjusting a Criterion benchmark alongside unit tests.

## On-disk format compatibility (do not break silently)

ATSC writes `.bro` payloads using **bincode v2** (`bincode = 2.0.0-rc.3`) with a shared config (`atsc/src/compressor/mod.rs::BinConfig::get()`). `.vsri` files are written by the `vsri` crate as a line-based text format (not bincode).

Implications:
- **Changing the shape/order of `Encode`/`Decode` structs is format-breaking**, even if Rust still compiles.
- High-risk types include (non-exhaustive): `atsc::data::CompressedStream`, `atsc::frame::CompressorFrame`, `atsc::header::CompressorHeader`, and any compressor frame types in `atsc/src/compressor/*` (e.g. `FFT`, `FrequencyPoint`).

Guardrails:
- If you need to evolve the format, prefer **explicit versioning** (header/version field) and **versioned decode paths** rather than “just changing the struct”.
- When adding fields, consider whether they must be encoded; if not, follow the existing pattern (e.g. `FFT` manually omits `error` from encoding).
- Add a **roundtrip test** that reads representative bytes and validates decode behavior.

## Performance expectations

ATSC is designed for **fast decompression** and “reasonable” compression throughput. When optimizing:
- Treat **allocations and per-sample conversions** in hot paths as first suspects.
- Use existing Criterion benches as baselines and add focused micro-benches when needed.
- Prefer changes that are **measurable** (bench evidence) and **local** (don’t destabilize format without intent).

