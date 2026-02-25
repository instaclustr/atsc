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

## Acceptance gate status (current run)

- Frame-format overhead requirement (`<= 5%`) on gold files: **PASS**
- Noop fallback behavior is now measurable via `noop_select_rate`: **PASS (instrumented)**
- Compression and decompression comparisons against v1 thresholds:
  - **NOT EVALUATED in this report** (v1 baseline values were not recomputed in this run)

## Notes

- These matrix runs used `--runs 1` for fast iteration during policy validation.
- For release/merge gating, rerun with a higher run count and include explicit v1 comparison tables.
