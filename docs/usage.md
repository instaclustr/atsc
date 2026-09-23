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
`-c/--compression-selection-sample-level` accepts 0 through 6. In `compress`,
`auto` evaluates every candidate on the full frame and picks the smallest one
that meets `--error`, so `-c` does not change the chosen codec.

BRO, WBRO, and CSV readers default to a 256 MiB input limit and a 33,423,360
sample limit. WBRO archive shape is validated before archived vectors are
deserialized. CSV readers bound individual records and report invalid UTF-8,
unequal record lengths, missing fields, and invalid values as input-format
errors instead of panicking. File open and read failures remain I/O errors.

`compress` is strict. Non-finite samples are rejected with their sample index,
and if the selected codec cannot meet `--error` the command fails with exit
code 4 and writes no output. The error metric is a mean absolute percentage
error, so it is undefined (NaN or infinite) for inputs containing zeros with
lossy codecs; the error message then suggests `--compressor auto` (unless
`auto` already failed), `rle`, or `noop`. Use [legacy mode](#legacy-compatibility) for 0.7's best-effort output.

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

The 0.7 root-level invocations remain supported:

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

Legacy compression keeps 0.7's best-effort semantics, unlike `compress`:

- Non-finite samples (NaN, infinity) are dropped before compression, and one
  warning line with the dropped count is printed to stderr per file. The
  restored series is shorter than the input by that count.
- When the codec misses `--error`, output is still written: a forced codec
  writes its bounded result, and `auto` writes the smallest candidate when no
  candidate meets the bound. Compare the restored data if the bound matters.
- `auto` selects codecs as 0.7 did. With `-c` 1 through 6 it picks the codec
  from the first 4096 (`-c 1`) down to 128 (`-c 6`) samples of each frame long
  enough to sample, and writes that codec's full-frame result even if it
  misses `--error` on the whole frame.

The remaining behavior is shared with the explicit commands and differs from
0.7:

- `-u` uses the bounded parser and decoder of `decompress`; corrupt BRO input
  fails with exit code 3 instead of panicking.
- Errors are printed to stderr and use the [exit codes](#exit-status) below;
  0.7 exited with `1` on any failure.
- Directory input is filtered by extension, processed once per file, and
  rejected before writing if derived output paths collide.
- WBRO and CSV input is validated and limited as described above.

File output naming (`.bro` for compression, `.wbro` for `-u`) is unchanged.

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

ATSC uses stable exit codes in both explicit and legacy modes (0.7 exited with
`1` on any failure):

- `0`: success
- `1`: file open/read/write I/O or output-serialization error
- `2`: command usage error, including mixed legacy/subcommand syntax
- `3`: BRO parse or decode error
- `4`: compression error (strict input or error-bound failure; in legacy mode,
  only input that is empty after dropping non-finite samples or needs more than
  255 frames)
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
