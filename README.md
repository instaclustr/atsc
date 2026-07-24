# ATSC - Advance Time Series Compressor

**NOTE:** This is still under development. Current status is unsupported!

## Table of Contents

1. [TL;DR;](#tldr)
2. [What is ATSC?](#what-is-atsc)
3. [When to use ATSC?](#when-to-use-atsc)
4. [Documentation](#documentation)
5. [Building ATSC](#building-atsc)
6. [ATSC Usage](#atsc-usage)
7. [Releases](#releases)
8. [Roadmap](#roadmap)
9. [Support Status](#support-status)

## TL;DR

The fastest way to test ATSC is with a CSV file!

1. Download the latest [release](https://github.com/instaclustr/atsc/releases)
2. Pick a CSV file from [tests folder](https://github.com/instaclustr/atsc/tree/main/atsc/tests/csv) (Those will have the expected internal format).  
3. Execute the following command:

    ```bash
    cargo run --release -- compress --csv <input-file>
    ```

4. You have a compressed timeseries!

## What is ATSC?

Advanced Time Series Compressor (in short: ATSC), is a configurable, *lossy* compressor that uses the characteristics of a time-series to create a function approximation of the time series.

This way, ATSC only needs to store the parametrization of the function and not the data.

ATSC draws inspiration from established compression and signal analysis techniques, achieving significant compression ratios.

In internal testing ATSC compressed from 46x to 880x the monitoring timeseries of our databases with a fitting error within 1% of the original time-series.

In some cases, ATSC would produce highly compressed data without any data loss (Perfect fitting functions).
ATSC is meant to be used with long term storage of time series, as it benefits from more points to do a better fitting.

The decompression of data is faster (up to 40x) vs a slower compression speed, as it is expected that the data might be compressed once and decompressed several times.

Internally ATSC uses the following methods for time series fitting:

* FFT (Fast Fourier Transforms)
* Constant
* Interpolation - Catmull-Rom
* Interpolation - Inverse Distance Weight
* RLE (Run Length Encoder)

For a more detailed insight into ATSC read the paper here: [ATSC - A novel approach to time-series compression](https://github.com/instaclustr/atsc/tree/main/paper/ATCS-AdvancedTimeSeriesCompressor.pdf)

ATSC input can be an internal format developed to process time series (WBRO), or a CSV. It outputs a compressed format (BRO). A CSV to WBRO format is available here: [CSV Compressor](https://github.com/instaclustr/atsc/tree/main/csv-compressor)

## When to use ATSC?

ATSC fits in any place that needs space reduction in trade for precision.
ATSC is to time series what JPG/MP3 is to image/audio.
If there is no need of absolute precision of the output vs the original input, you could probably use ATSC.

Example of use cases:

* In places where time series are rolled over, ATSC is a perfect fit. It would probably offer more space savings without any meaningful loss in precision.
* Time series that are under sampled (e.g. once every 20sec). With ATSC you can greatly increase sample rate (e.g. once per second) without losing space.
* Long, slow moving data series (e.g. Weather data). Those will most probably follow an easy to fit pattern
* Data that is meant to be visualized by humans and not machine processed (e.g. Operation teams). With such a small error, under 1%, it shouldn't impact analysis.

## Documentation

For full documentation please go to [Docs](https://github.com/instaclustr/atsc/tree/main/docs)

## Building ATSC

1. Clone the repository:

   ```bash
   git clone https://github.com/instaclustr/atsc
   cd atsc
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

## ATSC Usage

### Prerequisites

* Ensure you have [Rust 1.81 or newer](https://www.rust-lang.org/tools/install) installed on your system.

### Usage

ATSC accepts WBRO input, or CSV input with `--csv`, and writes BRO v1 without
changing the wire format. Prefer the explicit commands:

```text
atsc inspect <INPUT> [--json]
atsc verify <INPUT>
atsc compress <INPUT> [-o <OUTPUT>] [OPTIONS]
atsc decompress <INPUT> [-o <OUTPUT>]
```

Explicit-command options belong after their subcommand. Combining an explicit
subcommand with legacy root input/options is a usage error.

Examples:

```bash
atsc inspect metrics.bro
atsc inspect metrics.bro --json
atsc verify metrics.bro
atsc compress metrics.wbro --compressor rle
atsc compress metrics.csv --csv --fields=time,value -o metrics.bro
atsc decompress metrics.bro -o restored.wbro
```

Compression defaults to automatic codec selection with a 3% maximum error.
Directory input is supported for compression and decompression; ATSC snapshots
and filters the initial entries before processing each eligible input once.
`-o/--output` is only valid for a single input file.
WBRO bodies and CSV structure are validated and return typed input-format
errors; file open/read/write failures retain the I/O exit status.

Existing invocations remain available as compatibility mode and use the same
safe implementation:

```bash
atsc [OPTIONS] <INPUT>
atsc -u <INPUT>
```

For a legacy file named `inspect`, `verify`, `compress`, or `decompress`, use
`atsc -- <name>` or an explicit path such as `atsc ./inspect`. ATSC does not
guess command intent from filesystem existence.

Library callers can use bounded parsing and a reusable decoder directly:

```rust
use atsc::{data::CompressedStream, decoder::Decoder};

let bytes = std::fs::read("metrics.bro")?;
let stream = CompressedStream::try_from_bytes(&bytes)?;
let mut decoder = Decoder::new();
let values = decoder.decode_range(&stream, 1_000..2_000)?;
```

See [the usage guide](docs/usage.md) for all compression options, output naming,
JSON behavior, and stable exit codes.

## Releases

### v0.7 - 20/11/2024

* Added CSV Support
* Greatly improved documentation
* Improved Benchmark and testing
* Improved FFT compression
* Improved Polynomial compression
* Demo files and generation scripts
* Several fixes and cleanups

### v0.6 - 09/11/2024

* Internal release

### v0.5 - 30/11/2023

* Added Polynomial Compressor (with 2 variants)
* Created and Integrated a proper file type (wbro)
* Benchmarks of the different compressors
* Integration testing
* Several fixes and cleanups

## Roadmap

* Frame expansion (Allowing new data to be appended to existing frames)
* Dynamic function loading (e.g. providing more functions without touching the whole code base)
* Global/Per frame error storage
* Efficient error

## Support Status

Please see https://www.instaclustr.com/support/documentation/announcements/instaclustr-open-source-project-status/ for Instaclustr support status of this project