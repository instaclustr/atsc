# Changelog

## Unreleased

BRO v1 is unchanged: the same streams are written and decoded byte for byte.

### Breaking: WBRO format

- `wavbrro` 0.2.0 stores the WBRO body as an rkyv 0.8 archive (was rkyv 0.7).
  Bytes 4..8 of the 12-byte header are now a format marker: new files start
  with `WBRO0001WBRO`, and readers accept only the `0001` marker.
- Files with the `WBRO0000WBRO` header written by earlier releases (rkyv 0.7
  body) can no longer be read. `wavbrro` returns `Error::LegacyFormat` and the
  `atsc` CLI exits with the input-format code `5`, stating that the file must
  be regenerated. Any other marker is a format error.
- Migration: regenerate `.wbro` files from their source data (for example with
  `csv-compressor` or `wav2wbro`), or decompress existing `.bro` files with this
  release, which writes the new header. BRO files do not need migration.
- The `atsc` CLI now writes WBRO through `wavbrro::write::try_write_wavbrro_file`
  and the shared `wavbrro::write::FILE_HEADER` instead of its own copy of the
  header.

### Toolchain and dependencies

- The minimum supported Rust version is 1.88 and every crate uses edition 2024.
  Package metadata and dependency versions are inherited from the workspace,
  which uses resolver 3 and denies unsafe code.
- Removed unused dependencies: `average`, `median`, `regex`, `num-traits`,
  `hound`, `inverse_distance_weight`, and the `rand` dev-dependency from
  `atsc`; `claxon`, `symphonia`, `dtw_rs`, `regex`, `median`, and `chrono` from
  `tools`; `log` and `env_logger` from `wavbrro`. `tempfile` is now a
  dev-dependency, and `csv-compressor` uses `tempfile` instead of the
  deprecated `tempdir`.
- Upgraded rustfft 6.4, clap 4.6, csv 1.4, thiserror 2.0.20, serde 1.0.229,
  serde_json 1.0.151, log 0.4.34, env_logger 0.11.11, tempfile 3.27,
  chrono 0.4.45, criterion 0.8, and rkyv 0.8. bincode stays pinned to
  `=2.0.0-rc.3` and splines to 4.3.x until they are replaced.
- `rand` 0.8, `remove_dir_all`, and `tempdir` are gone from `Cargo.lock`.

### CI

- Workflows default to read-only `contents` permissions; only the release
  publishing job gets `contents: write`. Actions are pinned by SHA
  (actions/checkout v7.0.1, Swatinem/rust-cache v2.9.2), and pushes to `main`
  no longer cancel each other.
- cargo-deny checks advisories, bans, licenses, and sources with a new
  `deny.toml`; the unmaintained-bincode advisory (RUSTSEC-2025-0141) is ignored
  until bincode is replaced.
- Dependabot checks Cargo and GitHub Actions dependencies weekly.

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
    candidate when none does). Where 0.7 panicked because no candidate met
    the bound on the sample, 0.8 falls back to full-frame selection.
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
