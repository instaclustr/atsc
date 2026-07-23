# Usage

## Explicit commands

The subcommand interface is recommended for new scripts:

```text
atsc inspect <INPUT> [--json]
atsc verify <INPUT>
atsc compress <INPUT> [-o <OUTPUT>] [OPTIONS]
atsc decompress <INPUT> [-o <OUTPUT>]
```

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
container errors and malformed codec payloads.

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

Without `-o`, a file keeps its base name and receives the `.bro` extension.
For directory input, ATSC snapshots the initial entries and processes each
eligible `.wbro` file once, or each `.csv` file once when `--csv` is active.
`-o/--output` is only valid for a single input file.

### Decompress

```bash
atsc decompress metrics.bro
atsc decompress metrics.bro -o restored.wbro
```

Without `-o`, a file keeps its base name and receives the `.wbro` extension.
Directory input processes each initial `.bro` file once.

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

## Exit status

ATSC uses stable exit codes:

- `0`: success
- `1`: I/O or output-serialization error
- `2`: command usage error
- `3`: BRO parse or decode error
- `4`: bounded compression error
- `5`: WBRO/CSV input-format error

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
