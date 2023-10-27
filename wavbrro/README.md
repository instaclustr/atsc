# WAVBRRO

**NOTE:** This needs some serious work!

WAVBRRO is a change to the WAV format to be used to store timeseries.

It uses the groundwork in the RIFF (Resource Interchange File Format) for this. As such, the WAV format was picked for this.

- Replace the WAV tag with BRRO tag

https://en.wikipedia.org/wiki/RF64 Exists, but couldn't find any decent support. So, WAVBRRO mostly extends the Hound crate (https://codeberg.org/ruuda/hound) to do the work for us.

Generators of timeseries can output 64bit float samples, WAV doesn't support 64bit float, so we work around that has details in the SPEC section

## SPEC

The spec for WAVBRRO is the following:

Extension: .wbro

- If single channel, the data is integer or float and with the bithdepth indicated in the header. All information is extracted directly from the header.
- If it has 2 channels, then it is a 64bit int sequence. The channels must be of 32bit bitdepth and of integer type
- If it has 4 channels, then it is a 64bit float sequence. The channels must be of 16bit bitdepth and of integer type

### Reading/Writting WAVBRRO

If the data is from 8bit unsigned integer up to 32bit float/integer, the writing and reading process is the same as
any normal WAV file, no changes are needed.

For 64Bit bitdepth the write process is done the following way:

- For 64Bit signed integers:
  - channel 1: 0..32 bit,
  - channel 2: 32..64 bit,
- For 64Bit float:
  - channel 1 0..16 bit,
  - channel 2: 16..32 bit,
  - channel 3: 32..48 bit,
  - channel 4: 48..64 bit

### What it doesn't support

WAVBRRO doesn't support the following bitdepths

- unsigned 16,32 and 64 bits,
- signed 8 bit,
- Anything bigger than 64bit bitdepth
