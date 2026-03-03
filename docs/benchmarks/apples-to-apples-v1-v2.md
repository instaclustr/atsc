# Apples-to-Apples v1 vs v2 Comparison

- mode: `auto`
- error target: `1%`
- runs: `5` (median)
- build: `--release`
- inputs: normalized copies of v1 CSV fixtures (`cpu_utilization.csv`, `iowait.csv`)
- normalization: v1 parser requires header `timestamp,value` (fixtures use `time,value`)
- decompression caveat: v1 `csv-compressor -u` exits non-zero (panic after writing output); timing below reflects command runtime anyway

|file|samples|v1_size_bytes|v2_size_bytes|size_ratio_v2_over_v1|v1_ratio|v2_ratio|v1_compress_ms|v2_compress_ms|compress_ratio_v2_over_v1|v1_decompress_ms|v2_decompress_ms|decompress_ratio_v2_over_v1|v1_decompress_exit_codes|
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
|cpu_utilization.csv|2854|22928|22948|1.0009|0.9958|0.9949|80.996|58.127|0.7177|78.511|53.468|0.6810|[101]|
|iowait.csv|2891|3294|23244|7.0565|7.0213|0.9950|71.716|54.906|0.7656|72.086|52.118|0.7230|[101]|
