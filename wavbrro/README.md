# WAVBRRO

**NOTE:** This needs some serious work!

WAVBRRO is a based on the WAV format to be used to store raw timeseries data.

## SPEC

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

### Reading/Writting WAVBRRO

Check the tests in the `wavbrro.rs`

### What it doesn't support

WAVBRRO doesn't support the following bitdepths

- Windows OS
- Timestamps (VSRI will land here eventually)
- Anything other than f64
- Anything bigger than 64bit bitdepth

### Next steps

- Implement streaming read and write
- Seeking for a specific block
- Support other bitdepth
