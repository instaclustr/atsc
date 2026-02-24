# FORGE Fix Execution Brief (atsc-v2)

This document is the execution brief for FORGE to address review findings from the recent `atsc-v2` implementation review.

## Locked decisions

- Drop FFT-f64 support for v2 (remove feature/spec references).
- Drop IDW codec from v2 (remove codec, CLI option, dependency).
- Keep existing tracked `PLAN.md`, but do not introduce new `PLAN*.md` / `COMPLAIN*.md` files.
- Fuzzing is required in CI (short smoke run on branch pushes).

## Priority 0: critical fixes (must land first)

1. **C1 - fixed-width error fields**
   - Replace `usize` in wire-adjacent error variants with fixed-width integers (`u64`).
   - Update constructors and tests for safe conversions.

2. **C2/C4 - optimizer fallback safety**
   - Ensure `select_codec` has deterministic fallback behavior and never swallows errors.
   - Enforce finite-input handling inside `select_codec` (not only at higher wrappers).
   - Guarantee Noop fallback path for valid finite data.

3. **C3 - remove double decode**
   - In timestamp decompression flow, decode stream once and reuse header/frames.

4. **C5 - FFT heap float ordering safety**
   - Prevent non-finite spectrum values from entering heap ordering path.
   - Add guard/assert + test.

5. **C6 - DataStats contract hardening**
   - Reject non-finite input in `DataStats` (preferred) or constrain visibility with explicit checks at all call sites.
   - Add NaN/Inf tests.

## Priority 1: high-impact concerns

6. **W1 - remove duplicated wire helpers**
   - Introduce shared internal read/write helpers and replace copy-pasted functions.

7. **W2/W3 - chunking/FFT edge guards**
   - Fix `prev_power_of_two` portability issue.
   - Add sanity guard in `next_smooth` for sentinel failure case.

8. **W5 - remove unchecked count truncation**
   - Replace `as u32` frame count casts with checked conversions.

9. **W7 - clearer timestamp filtering behavior**
   - Improve error for value/timestamp mismatch after filtering (avoid strict-mode wording confusion).

10. **W8 - remove dead v1 tree**
    - Delete unused legacy v1 modules from active crate tree.
    - Remove stale dependencies if no longer needed.

11. **W10 - add fuzz targets**
    - Add fuzz harnesses for `decompress`, `decode_stream`, `decode_header`.
    - Wire short fuzz smoke run into CI.

## Priority 2: spec alignment and quality

12. **Drop FFT-f64**
    - Remove feature flag, implementation stubs, and references in docs/spec.
    - Mark codec ID 3 as deprecated in format documentation.

13. **Drop IDW**
    - Remove codec and selection paths.
    - Remove CLI `idw` option.
    - Remove `inverse_distance_weight` dependency.

14. **Polynomial payload cleanup (W4)**
    - Either implement promised bitdepth variants fully or simplify payload format and document accordingly.
    - Keep wire-compat behavior explicit.

15. **Suggestions**
    - Add `#[non_exhaustive]` to public `Error`.
    - Add compression ratio helper.
    - Expand property/corrupt-input tests to all active codecs.
    - Keep Constant candidate available for near-constant data paths.

## Commit discipline for FORGE

- One logical change per commit.
- Mandatory pre-commit checks:
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- If touching CI/fuzz, ensure CI config and local command docs stay consistent.

## Suggested commit order

1. C1
2. C3
3. C2/C4
4. C5
5. C6
6. W1
7. W2/W3
8. W5
9. W7
10. W8
11. W10
12. Drop FFT-f64
13. Drop IDW
14. W4 payload alignment
15. Suggestions/tests polish
