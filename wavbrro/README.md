# WAVBRRO

**NOTE:** This library is under development. It might have breaking changes.

WAVBRRO is a based on the WAV format to be used to store raw timeseries data.

## WAVBRRO SPECIFICATION

The spec for WAVBRRO is the following:

Extension: .wbro

- Header, 12 Bytes
  - 0..3 "WBRO"
  - 4..7 Sample number (u32)
  - 8..12 "WBRO"
- Internal Structure
  - Sample number: u32
  - Bitdepth: u8 [0 -> u8, 1 -> i16, 2 -> i32, 3 -> i64, 4 -> f32, 5 -> f64]
  - Samples
    - Blocks of 2048 samples

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
