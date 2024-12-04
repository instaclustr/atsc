# ATSC - Clickhouse integration build

**NOTE**: This is not in sync with the latest release.

The ATSC compressor hasn't been integrated into the official ClickHouse repository. However, you can try it out using our repository at Instaclustr. To do so, follow the link below and clone the  following fork: <https://github.com/instaclustr/ClickHouse/tree/rust-brro-success-without-bridge-backup>

Once cloned, you can build both the ClickHouse server and client. The build and setup process is identical to the official documentation: (<https://clickhouse.com/docs/en/development/developer-instruction>). After building, you will have ClickHouse running with the integrated ATSC compressor.

Once the build is done creating tables can be done the following way:

```sql
CREATE TABLE sensors_atsc (  
    sensor_id UInt16,  
    location UInt32,  
    timestamp DateTime,  
    pressure Float64 CODEC(BRRO('Auto')),  
    temperature Float64 CODEC(BRRO('Auto', 3)),  
) 
ENGINE = MergeTree  
ORDER BY (sensor_id, location, timestamp);
```

## Major Changes

### 0.5

- Added Polynomial Compressor (with 2 variants)
- Created and Integrated a proper file type (wbro)
- Benchmarks of the different compressors
- Integration testing
- Several fixes and cleanups

## Description

BRRO Compressor is a compressor that relies on the characteristics of a signal to provide a far greater compression that currently existing ones. BRRO relies on different techniques based on a initial analysis of the signal to use the best suited method for compressor for that specific signal segment.

For a detailed description on the compressor methods and logic check `BRRO.md`.

## Getting Started with BRRO Compressor

### Prerequisites

- Ensure you have [Rust](https://www.rust-lang.org/tools/install) and Cargo installed on your system.

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/instaclustr/fft-compression
   cd fft-compression
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

### Usage

Currently BRRO relies on have Raw BRRO files generated by our prometheus remote endpoint. This would work as input for the compressor.

Compressor usage:

```
Usage: brro-compressor [OPTIONS] <INPUT>

Arguments:
<INPUT>  input file

Options:
--compressor <COMPRESSOR>  [default: auto] [possible values: auto, fft, constant, polynomial]
-u                             Uncompresses the input file/directory
-h, --help                     Print help
-V, --version                  Print version
```

#### Compress a File

To compress a file using the BRRO Compressor, run:

```bash
brro-compressor <input-file>
```

#### Decompress a File

To decompress a file, use the following command:

```bash
brro-compressor -u <input-file>
```

## Programs and description

This repository contains one main program and other programs that serve different purposes, some are for just testing, others do some actual work.

### flac-server

**NOTE**: Remote read is currently NOT working, as it depends on FLAC files that are no longer generated.

Needs a prometheus server. We need it to get our samples out. Supports read and write from prometheus.

Launch the `flac-server` and set it as your remote endpoint for prometheus, example below.

```YAML
# Remote read and Write
remote_write:
   - url: "http://localhost:9201/api/write"

remote_read:
   - url: "http://localhost:9201/api/read"
     read_recent: true
     name: "flac_server"
```

Make Prometheus server a source of your grafana and check the data.

### brro_optimizer

Maybe the most important tool at this point, it picks a WAV file from the datasets described below and optimizes it into a way that we might see a meaningful compression into FLAC.
The tool also has options to dump the output of the file as a single sample per period, instead of the 4 channels. This is good to obtain the data as it was feed into the flac-server.
The code performs optimizations based on file name, so renaming might cause issues.

Usage (Getting raw samples): `./brro_optimizer infile.wav --dump-raw > file.raw`
Usage (Getting optimized samples): `./brro_optimizer infile.wav --dump-optimized > file.raw`
Usage (Generate a optimized file): `./brro_optimizer -w infile.wav`

If you set the ENV Variable for Debug it will output what it is doing.

### Matlab folder

Exploratory code. Should be removed.

## Roadmap

1. Update `flac-server` to read/write WBRO/BRO files.
2. Streaming compression/decompression
3. Automated compressor selection
4. Frame expansion (Allowing new data to be appended to existing frames)
