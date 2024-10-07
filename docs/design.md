BRRO
--

The concept behind BRRO is to process time-series data the way usually multimedia data is processed. It allows
to reach better compression ratios than typical compression algorithms that are used for
time-series data (e.a Gorilla).

General writing flow:
1. Take time-series data
2. Values is being translated into WavBRROs and time is being translated into VSRI
3. Apply compression (e.a. FFT) on WavBRRO
4. As a result we have compressed data (.bro) and indexes for it (.vsri)

General reading flow:
1. Decompressing .bro data using brro-compressor. The compression algorithm that was applied on the specific chunk of data is
   encoded at the beginning of the frame so brro-compressor knows which algorithm should be used to decompress data.
2. Using VSRI to understand which samples from decompressed data should be returned.
3. Assembling retrieved samples with the corresponding timestamps from the index.
4. As a result it returns time-series data

For more details on BRRO, including what it is and the concept behind it, you can refer to this [paper](../paper/BRRO.md).

BRRO Compressor
--

BRRO Compressor is designed to compress WavBRRO formatted time-series data. It provides such compression algorithms as:
1. Auto
2. Noop
3. Fast Fourier Transform (FFT)
4. Polynomial
5. Constant 
6. Idw

If compressor is set to **auto** then BRRO compressor will decide which of the algorithm it should use to compress
some special data chunk.

Compressor usage:

```
Usage: brro-compressor [OPTIONS] <INPUT>

Arguments:
  <INPUT>  input file

Options:
      --compressor <COMPRESSOR>
          Select a compressor, default is auto [default: auto] [possible values: auto, noop, fft, constant, polynomial, idw]
  -e, --error <ERROR>
          Sets the maximum allowed error for the compressed data, must be between 0 and 50. Default is 5 (5%). 0 is lossless compression 50 will do a median filter on the data. In between will pick optimize for the error [default: 5]
  -u
          Uncompresses the input file/directory
  -c, --compression-selection-sample-level <COMPRESSION_SELECTION_SAMPLE_LEVEL>
          Samples the input data instead of using all the data for selecting the optimal compressor. Only impacts speed, might or not increased compression ratio. For best results use 0 (default). Only works when compression = Auto. 0 will use all the data (slowest) 6 will sample 128 data points (fastest) [default: 0]
      --verbose
          Verbose output, dumps everysample in the input file (for compression) and in the ouput file (for decompression)
  -h, --help
          Print help
  -V, --version
          Print version
```

CSV Compressor
--

CSV Compressor allows compressing CSV formatted time-series data. It leverages BRRO Compressor functionalities to compress
data.

Compression flow:
1. Reads provided time-series data as a CSV
2. Transforms values of time-series data into WavBRRO and generate VSRI
3. Compresses achieved WavBRRO

Decompression flow:
1. Read compressed WavBRRO (we call it **bro**)
2. Decompresses data
3. Read VSRI and retrieves timestamps
4. Outputs time-series data as CSV

In the current state in only generates a **single** WavBRRO, BRO and VSRI files which contain time-series data.

CSV Compressor usage:

```
Usage: csv-compressor [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to input

Options:
  -o, --output <OUTPUT>
          Defines where the result will be stored
  -u
          Defines if we should uncompress input
      --no-compression
          
      --output-vsri
          Enables output of generated VSRI
      --output-wavbrro
          Enables output of generated WavBrro
      --output-csv
          Enable output result of decompression in CSV format
      --compressor <COMPRESSOR>
          Select a compressor, default is auto [default: auto] [possible values: auto, noop, fft, constant, polynomial, idw]
  -e, --error <ERROR>
          Sets the maximum allowed error for the compressed data, must be between 0 and 50. Default is 5 (5%). 0 is lossless compression 50 will do a median filter on the data. In between will pick optimize for the error [default: 5]
  -c, --compression-selection-sample-level <COMPRESSION_SELECTION_SAMPLE_LEVEL>
          Samples the input data instead of using all the data for selecting the optimal compressor. Only impacts speed, might or not increased compression ratio. For best results use 0 (default). Only works when compression = Auto. 0 will use all the data (slowest) 6 will sample 128 data points (fastest) [default: 0]
  -h, --help
          Print help
  -V, --version
          Print version
```

WavBRRO
--
WavBRRO lib crate contains an implementation of WavBRRO format. The format is a based on the WAV format to be used to 
store raw time-series data.

For more details on WavBRRO you may follow [here](../wavbrro/README.md).

Vsri
--
Vsri lib crate contains an implementation of the VSRI (Very Small Rolo Index). The index is made for detection of gaps 
in continuous data with the same sampling rate.

Each continuous segment of data will be mapped to a line using the formula y = mx + B plus the number of points in 
the data series, where:
- m - Sampling rate
- b - Series initial point in time in [x,y]
- x - sample # in the data file, this is ALWAYS sequential. There are no holes in samples
- y - time

This way, discovering the segment number is solving the above equation for X if the time provided is bigger than 
the initial point.

Index structure:
1. index_name: Name of the index file we are indexing
2. min_ts: the minimum TS available in this file
3. max_ts: the highest TS available in this file
4. vsri_segments: Description of each segment:
    1. Sampling rate
    2. initial sample position X0
    3. initial sample timestamp Y0
    4. Number of samples in the segment

Example of content of an index:

     55745
     59435
     15,0,55745,166
     15,166,58505,63

Where:

- 55745 - min_ts
- 59435 - max_ts
- 15,0,55745,166 - the first segment:
    1. 15 - Sampling rate
    2. 0 - initial sample position for the segment X0
    3. 55745 - initial sample timestamp for the segment Y0
    4. 166 - the number of sampels in the first segment
- 15,166,58505,63 - the second segment:
    1. 15 - Sampling rate
    2. 166 - initial sample position for the segment X0
    3. 58505 - initial sample timestamp for the segment Y0
    4. 63 - the number of sampels in the first segment
