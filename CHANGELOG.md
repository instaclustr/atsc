# Changelog

## 0.8.0

BRO v1 stays wire compatible: streams written by 0.7 decode unchanged, and
0.8 writes the same BRO v1 layout, bincode configuration, and codec IDs.

### Library

- New strict APIs return typed errors instead of panicking:
  `CompressedStream::try_from_bytes`/`try_from_bytes_with_limits`,
  `try_decompress`, `try_compress_chunk_bounded_with`,
  `CompressorFrame::try_compress_bounded`/`try_compress_best`,
  `Compressor::try_compress_bounded`/`try_decompress`, and
  `OptimizerPlan::try_plan`. Strict encoders reject empty or non-finite
  input and missed error bounds without mutating the stream.
- New `Decoder` for reusable full, per-frame, and sample-range decoding, plus
  `CompressedStream::frame_info`.
- New `error::DecodeError`, `error::EncodeError`, and `error::DecodeLimits`
  (default 256 MiB input, 255 frames, 33,423,360 samples).
  `atsc::csv::CsvReadLimits` and `wavbrro::wavbrro::ReadLimits` bound CSV and
  WBRO reads.
- Legacy infallible APIs keep 0.7 best-effort semantics:
  - `OptimizerPlan::plan` drops non-finite samples and panics only when no
    samples remain.
  - `compress_chunk_bounded_with`, `CompressorFrame::compress_bounded`/
    `compress_best`, and `Compressor::compress_bounded` still commit the
    forced codec's output when the error bound is missed, and Auto selects
    codecs as 0.7 did (from the sample prefix when a sampling speed is set,
    otherwise the smallest candidate meeting the bound, or the smallest
    candidate when none does).
  - They now panic, before mutating the stream, on empty or non-finite chunks
    and at the 255-frame BRO v1 limit instead of writing a corrupt frame
    count.
- Strict Auto (`try_compress_best`, `try_compress_chunk_bounded_with`)
  evaluates every candidate on the full frame and never picks one that misses
  the bound, so in strict mode the sampling speed
  (`-c/--compression-selection-sample-level`) does not change the selected
  codec.

### CLI

- New subcommands: `compress`, `decompress` (both with `-o/--output`),
  `inspect` (with `--json` for one machine-readable document), and `verify`.
  `compress` is strict: non-finite samples and missed error bounds fail with a
  typed error and no output file. When the error is undefined (NaN or
  infinite) because the input contains zeros, the message says so and
  suggests `--compressor auto` (unless Auto already failed), `rle`, or `noop`.
- Legacy root syntax (`atsc [OPTIONS] <INPUT>`, `atsc -u <INPUT>`) keeps
  best-effort compression: non-finite samples are dropped with one warning
  line on stderr, a missed error bound still writes output, and Auto selection
  (including `-c`) reproduces 0.7's output.
- Errors are printed to stderr and use stable exit codes: `1` I/O, `2` usage,
  `3` decode, `4` encode, `5` input format. 0.7 exited `1` on any failure.
- Directory input only processes files with the input extension (`.wbro`,
  `.csv` with `--csv`, `.bro` for decompression), processes each file once,
  and rejects inputs whose derived output paths collide before writing.
- WBRO and CSV input is validated and size-limited; malformed files are
  reported as input-format errors instead of panicking.
- Mixing legacy root options with a subcommand is a usage error; use
  `atsc -- <name>` for legacy files named like a subcommand.

### Build

- Minimum supported Rust version is 1.81 (declared as `rust-version`).
