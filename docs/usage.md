# Usage

## Explicit commands

The subcommand interface is recommended for new scripts:

```text
atsc inspect <INPUT> [--json]
atsc verify <INPUT>
atsc compress <INPUT> [-o <OUTPUT>] [OPTIONS]
atsc decompress <INPUT> [-o <OUTPUT>]
```

Options for an explicit command must be placed in that command's argument
scope, for example `atsc compress --compressor rle metrics.wbro`. Legacy root
inputs or options cannot be combined with an explicit subcommand; mixed syntax
is a usage error.

### Inspect a BRO stream

`inspect` parses the bounded BRO container and reports its version, frame and
sample counts, and per-frame codec and payload size. It does not decompress
frame payloads.

```bash
atsc inspect metrics.bro
atsc inspect metrics.bro --json
```

`--json` writes exactly one JSON document to stdout. Logs and errors are
written to stderr.

### Verify a BRO stream

`verify` parses the container and fully decodes every frame, so it detects both
container errors and malformed codec payloads, including non-finite
reconstruction parameters.

```bash
atsc verify metrics.bro
```

A valid stream prints `valid`.

### Compress

```bash
atsc compress metrics.wbro
atsc compress metrics.wbro -o archive/metrics.bro
atsc compress metrics.wbro --compressor fft --error 1
atsc compress metrics.csv --csv --fields=time,value
atsc compress values.csv --csv --no-header
```

Compression defaults to `--compressor auto` and `--error 3`. Available codecs
are `auto`, `noop`, `fft`, `constant`, `polynomial`, `idw`, and `rle`.
`-c/--compression-selection-sample-level` accepts 0 through 6.

BRO, WBRO, and CSV readers default to a 256 MiB input limit and a 33,423,360
sample limit. WBRO archive shape is validated before archived vectors are
deserialized. CSV readers bound individual records and report invalid UTF-8,
unequal record lengths, missing fields, and invalid values as input-format
errors instead of panicking. Non-finite compression inputs are rejected with
their sample index. File open and read failures remain I/O errors.

Without `-o`, a file keeps its base name and receives the `.bro` extension.
For directory input, ATSC snapshots the initial entries and processes each
eligible `.wbro` file once, or each `.csv` file once when `--csv` is active.
All input/output pairs are derived first; if two eligible names would produce
the same `.bro` path, no file is processed.
`-o/--output` is only valid for a single input file.

### Decompress

```bash
atsc decompress metrics.bro
atsc decompress metrics.bro -o restored.wbro
```

Without `-o`, a file keeps its base name and receives the `.wbro` extension.
Directory input processes each initial `.bro` file once.
Duplicate derived `.wbro` paths are rejected before any output is written.

## Legacy compatibility

Existing root-level invocations remain supported:

```text
atsc [OPTIONS] <INPUT>
atsc -u <INPUT>
```

For example:

```bash
atsc --compressor rle metrics.wbro
atsc --csv --fields=time,value metrics.csv
atsc --csv --no-header values.csv
atsc -u metrics.bro
```

Legacy mode uses the same bounded, fallible implementation as `compress` and
the same safe parser and decoder as `decompress`. Existing file and directory
output naming is unchanged.

The four subcommand names are reserved command words. Disambiguate a legacy
file with one of the standard path forms; ATSC does not guess based on whether
a file happens to exist:

```bash
atsc -- inspect
atsc ./inspect
```

The same forms apply to legacy files named `verify`, `compress`, or
`decompress`.

## Exit status

ATSC uses stable exit codes:

- `0`: success
- `1`: file open/read/write I/O or output-serialization error
- `2`: command usage error, including mixed legacy/subcommand syntax
- `3`: BRO parse or decode error
- `4`: bounded compression error
- `5`: malformed WBRO header/body or CSV UTF-8/shape/field/value error

For a directory with multiple failures, every eligible entry is attempted
once and the highest applicable failure code is returned.

## Rust library

Use the bounded parser and a caller-owned decoder for reusable full, frame, or
range decoding:

```rust
use atsc::{data::CompressedStream, decoder::Decoder};

let bytes = std::fs::read("metrics.bro")?;
let stream = CompressedStream::try_from_bytes(&bytes)?;
let mut decoder = Decoder::new();
let values = decoder.decode_range(&stream, 1_000..2_000)?;
```

`wavbrro::wavbrro::ReadLimits` and `atsc::csv::CsvReadLimits` expose the same
byte/sample controls for library callers. Their existing convenience readers
remain default-limit wrappers, while CSV value-only readers avoid building an
intermediate `Vec<Sample>`.

BRO v1's `frame_size` field is opaque metadata produced from a historical
host-layout estimate. It is not validated or used for decoding; a real frame
byte length belongs in BRO v2.
