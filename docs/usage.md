# Usage

## CLI Options

Compressor usage:

```bash
Usage: atsc [OPTIONS] <INPUT>

Arguments:
  <INPUT>  input file

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

## Examples

### Compressing a file with a specific compressor

When this should be used?

When data in known and this way, avoid sample analysis and compress faster.

```bash
atsc --compressor fft <input-file> 
```

### Compressing a file with a specific error level

When this should be used?

When it is necessary to restrict the error of the output data.

```bash
atsc -e 1 <input-file> 
```

### Compressing a file with a specific compressor and a specific error level

When this should be used?

When data in known and this way, avoid sample analysis and compress faster and restrict the error of the output data.

```bash
atsc --compressor fft -e 1 <input-file> 
```

### Improving compression speed by reducing sample analysis

When this should be used?

There is enough knowledge about the data structure (e.g. it often repeats) that using a reduced sample size is ok.
The worst case would be a less than ideal compressor selected, impacting the compression ration.

```bash
atsc -c 6 <input-file>
```

### Decompressing

When this should be used?

If the data is needed!

```bash
atsc -u <input-file> 
```
