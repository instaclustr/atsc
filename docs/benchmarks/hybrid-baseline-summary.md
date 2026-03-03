# Hybrid Residual Baseline Summary

This summary captures focused baseline findings for the `hybrid-residual` codec.

## Run configuration

- command:
  - `cargo run -p atsc-cli --features perf-telemetry -- baseline /tmp/hybrid-real.f64 --runs 1 --output docs/benchmarks/hybrid-baseline-matrix.md`
- matrix output:
  - `docs/benchmarks/hybrid-baseline-matrix.md`
- notes:
  - `/tmp/hybrid-real.f64` is deterministic synthetic real input created for baseline mode.

## Key findings

- Hybrid bound hit rate:
  - `12/64` matrix cells met the requested error bound.
- Dataset-level hit rates:
  - `smooth-trend`: `12/16`
  - `periodic`: `0/16`
  - `noisy-entropy`: `0/16`
  - `mixed-realworld`: `0/16`
- Forced-mode payload winner count:
  - `forced-hybrid` won `0` cells when compared to bound-meeting `forced-fft`/`forced-poly`.

## Derived policy

- Hybrid is useful mainly on very smooth chunks; it underperforms on periodic/high-entropy/mixed chunks.
- Added bounded fallback threshold in codec:
  - `HYBRID_BOUNDED_ROUGHNESS_MAX = 0.0015`
- Operational effect:
  - bounded hybrid exits early with best-effort baseline payload on rough chunks,
  - avoids spending extra work on chunk shapes where hybrid does not meet bounds in this benchmark slice.
