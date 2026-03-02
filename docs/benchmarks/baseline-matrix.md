# ATSC Baseline Matrix Report (Speed-Size Convergence)

This report summarizes the strict-vs-default bounded-auto comparison required by the speed-size convergence process.

## Inputs and commands

- Source files (from `atsc/tests/wbros/`):
  - `go_gc_heap_goal_bytes.wbro`
  - `memory_used.wbro`
  - `uptime.wbro`
- Converted to raw f64 with `tools/src/bin/wbro2f64.rs`.
- Combined input used for matrix runs: `atsc/tests/f64/gold-combined.f64`.
- Matrix runs:
  - default policy: `strict_bound=false`
  - strict policy: `strict_bound=true`
  - command: `cargo run -p atsc-cli --features perf-telemetry -- baseline ... --runs 1`

Full generated matrices:
- `docs/benchmarks/baseline-matrix-default.md`
- `docs/benchmarks/baseline-matrix-strict.md`

## Telemetry column updates

The auto matrix now includes:
- `noop_select_rate`
- existing `attempts_avg`, `attempts_p95`, `retries`, `bound_miss_rate`, and `codec_time_share`

Per-chunk telemetry now includes explicit fallback reason classification (`none`, `lossy_best_effort`, `noop_safety`).

## Auto-mode strict vs default summary

Global averages across all auto cells (64 cells):

|metric|default|strict|strict/default|
|---|---:|---:|---:|
|auto median_time_ms (avg)|1252.1757|1276.7671|1.0196x|
|auto ratio (avg)|33.0781|32.9321|0.9956x|
|auto nrmse (avg)|0.007426|0.004742|0.6383x|
|auto noop_select_rate (avg)|0.0000|0.0436|n/a|
|auto noop_select_rate (max)|0.0000|0.5000|n/a|

Dataset-level auto averages:

|dataset|default_time_ms|strict_time_ms|strict/default|default_ratio|strict_ratio|default_noop_rate|strict_noop_rate|
|---|---:|---:|---:|---:|---:|---:|---:|
|mixed-realworld|1689.1511|1657.8365|0.9815x|18.4641|17.8840|0.0000|0.1699|
|noisy-entropy|2499.9797|2597.8494|1.0391x|1.0012|0.9973|0.0000|0.0045|
|periodic|443.2835|454.8101|1.0260x|50.8438|50.8438|0.0000|0.0000|
|smooth-trend|376.2886|396.5724|1.0539x|62.0033|62.0033|0.0000|0.0000|

## Gold file spot check (auto, 1% error, 22 iterations)

### Default mode (`strict_bound=false`)

|file|compress_ms|decompress_ms|ratio|encoded_bytes|frame_count|overhead_bytes (`14 + 9*F`)|overhead_pct|
|---|---:|---:|---:|---:|---:|---:|---:|
|go_gc_heap_goal_bytes.wbro|14.728|4.201|0.9951|23740|3|41|0.1727%|
|memory_used.wbro|11.718|3.111|0.9960|18482|2|32|0.1731%|
|uptime.wbro|4.602|2.404|8.8119|2089|2|32|1.5318%|

### Strict mode (`strict_bound=true`)

|file|compress_ms|decompress_ms|ratio|encoded_bytes|frame_count|overhead_bytes (`14 + 9*F`)|overhead_pct|
|---|---:|---:|---:|---:|---:|---:|---:|
|go_gc_heap_goal_bytes.wbro|12.768|3.539|0.9951|23740|3|41|0.1727%|
|memory_used.wbro|9.961|3.515|0.9960|18482|2|32|0.1731%|
|uptime.wbro|2.459|1.648|8.8119|2089|2|32|1.5318%|

## v1 vs v2 comparison (current code base)

Method:
- v1 reference: tag `v0.7.1` (`/home/crolo/code/fft-compression-v1`)
- v2 reference: current `atsc-v2`
- mode: auto, error=1%
- timing: median of 5 runs per file
- ratio columns below are `v2 / v1` (lower is better)

|file|v1_size_bytes|v2_size_bytes|size_ratio_v2_over_v1|v1_compress_ms|v2_compress_ms|compress_ratio_v2_over_v1|v1_decompress_ms|v2_decompress_ms|decompress_ratio_v2_over_v1|
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|go_gc_heap_goal_bytes.wbro|5021|23740|4.7281|3.680|3.139|0.8530|1.472|1.557|1.0579|
|memory_used.wbro|11566|18482|1.5980|4.691|2.738|0.5837|1.536|1.347|0.8769|
|uptime.wbro|73|2089|28.6164|1.351|1.287|0.9523|1.162|1.022|0.8790|

## Acceptance gate status (current run)

- Frame-format overhead requirement (`<= 5%`) on gold files: **PASS**
- Noop fallback behavior is now measurable via `noop_select_rate`: **PASS (instrumented)**
- Compression size ratio gate (`<= 1.05x v1` on gold files): **FAIL** (`4.7281x`, `1.5980x`, `28.6164x`)
- Compression time gate (`<= 1.50x v1` on gold files): **PASS** (`0.8530x`, `0.5837x`, `0.9523x`)
- Decompression time gate (`<= 1.00x v1` on gold files): **FAIL** (`1.0579x` on `go_gc_heap_goal_bytes.wbro`)

## Root cause and post-change sweep (1/3/5%)

Why auto previously picked `poly` too often:
- bounded probe miss path prioritized lower error over payload and returned on first full-budget bound-met winner.
- this favored polynomial on these datasets even when FFT payload was materially smaller.

Post-change sweep (median of 3 runs, v1 auto as denominator):

|error|file|v2 auto / v1 auto|v2 fft / v1 auto|v2 poly / v1 auto|
|---:|---|---:|---:|---:|
|1%|go_gc_heap_goal_bytes.wbro|5.5248x|0.5911x|5.5248x|
|1%|memory_used.wbro|1.5980x|0.1837x|1.5980x|
|1%|uptime.wbro|46.4222x|1.9778x|50.7111x|
|3%|go_gc_heap_goal_bytes.wbro|4.1268x|0.6176x|4.1268x|
|3%|memory_used.wbro|3.4852x|0.4007x|3.4852x|
|3%|uptime.wbro|46.4222x|1.9778x|50.7111x|
|5%|go_gc_heap_goal_bytes.wbro|1.2260x|0.7677x|3.2081x|
|5%|memory_used.wbro|2.6667x|1.1579x|8.5603x|
|5%|uptime.wbro|16.5556x|1.9778x|20.8444x|

Takeaway:
- auto now tracks a size-first objective better when both lossy codecs meet bound (notably `go_gc_heap_goal_bytes` at 5%).
- FFT remains consistently smaller than Polynomial on all three files at 1/3/5%, so additional policy bias toward FFT for these dataset shapes is still warranted.

## Notes

- These matrix runs used `--runs 1` for fast iteration during policy validation.
- For release/merge gating, rerun with a higher run count (for example 15+) and include variance/p95.
