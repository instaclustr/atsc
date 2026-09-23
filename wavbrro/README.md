# WAVBRRO

**NOTE:** This library is under development. It might have breaking changes.

WAVBRRO is a based on the WAV format to be used to store raw timeseries data.

## WAVBRRO SPECIFICATION

The spec for WAVBRRO is the following:

Extension: .wbro

- Header, 12 Bytes: `WBRO0001WBRO`
  - 0..4 "WBRO"
  - 4..8 Format marker, ASCII `0001`
  - 8..12 "WBRO"
- Body: an [rkyv](https://rkyv.org) 0.8 archive (little-endian, aligned,
  32-bit relative pointers) validated with bytecheck before any sample vector is
  materialized
  - Sample number: u32
  - Bitdepth: u8 [0 -> u8, 1 -> i16, 2 -> i32, 3 -> i64, 4 -> f32, 5 -> f64];
    only 5 (f64) is accepted
  - Samples
    - Blocks of 2048 samples; only the final block may be shorter, and it holds
      at least one sample
    - The block lengths must sum to the sample number; an empty file has no blocks

Readers accept only the `0001` marker. Files written by wavbrro 0.1 carry the
marker `0000` and an rkyv 0.7 body, which is no longer readable; reading one
returns `Error::LegacyFormat`. Regenerate such files from the source data (for
example with `csv-compressor` or `wav2wbro`). Any other marker returns
`Error::FormatError`.

### WAVBRRO API

#### Writing a file

```rust
    fn write_wavbrro() {
        // Create a temporary directory for the file
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let path = temp_dir.path().join("test.wbro");
        // Creating the structure
        let mut wb = WavBrro::new();
        // Adding samples
        wb.add_sample(1.0);
        wb.add_sample(2.0);
        wb.add_sample(3.0);
        wb.to_file(&path);
        let result = is_wavbrro_file(&path);
        assert!(result.unwrap());
    }
```

#### Reading a file

```rust
    fn read_wavbrro() {
        let data = WavBrro::from_file(path);
    }
```

Check the tests in the `wavbrro.rs` for more examples.

### What it doesn't support

WAVBRRO doesn't support the following bitdepths

- Windows OS
- Timestamps ([VSRI](https://github.com/instaclustr/atsc/tree/main/vsri) in here?)
- Anything other than f64
- Anything bigger than 64bit bitdepth

### Next steps

- Implement streaming read and write
- Seeking for a specific block
- Support other bitdepths
