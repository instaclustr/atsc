# Hybrid Residual Point Strategy

This document defines the decision model used by the `hybrid-residual` codec.

## Objective

For each chunk, compare two ways to reduce error:

- increase base model complexity;
- keep a simple base model and add sparse residual points.

The codec uses:

- `B_base(c)`: bytes for baseline complexity `c`;
- `B_patch(k)`: bytes for `k` residual patch points;
- `E(c, k)`: final NRMSE.

Patch points are preferred when:

- `delta_error_per_byte_patch > delta_error_per_byte_complexity`; or
- for an error target `E*`, `B_base(low_c) + B_patch(k*) < B_base(high_c)`.

## Default patch budget policy

- absolute cap:
  - `k_abs_cap = 32` when chunk size `<= 2048`;
  - `k_abs_cap = 64` when chunk size `> 2048`.
- relative cap:
  - `k_rel_cap = ceil(0.01 * N)`.
- effective cap:
  - `k_cap = min(k_abs_cap, k_rel_cap)`.

Candidate set is geometric and early-stop friendly:

- `k in {0, 4, 8, 16, 32, 64}` with `k <= k_cap`;
- include `k_cap` if it is not already in the set.

## No-big-rounds approach

The implementation avoids repeated base re-fitting:

1. fit baseline once;
2. reconstruct once;
3. rank residual magnitudes once;
4. use prefix sums of squared residuals to estimate `E(c, k)` quickly;
5. estimate payload bytes analytically from base sample count + varint index deltas + quantized residual payload.

This allows bounded selection over multiple `k` candidates with one base fit.
