# ATSC v2 — Agent Operating Rules

This file governs how AI agents work on the `atsc-v2` branch. Every agent session must follow these rules without exception.

## Commit Discipline

- **One logical change per commit.** Never bundle unrelated changes. A "change" is one of: a new module, a bug fix, a refactor, a test addition, a dependency change.
- **Every commit must pass `cargo test --workspace` before it is created.** Run tests, confirm green, then commit.
- **Every commit must pass `cargo clippy --workspace --all-targets` with no warnings.** Fix all clippy findings before committing.
- **Run `cargo fmt` before every commit.** Never commit unformatted code.
- **Commit message format:** imperative mood, first line under 72 chars, blank line, then body explaining *why* (not *what*).

```
Add NRMSE metric with constant-signal special case

NRMSE replaces the buggy MAPE metric. Returns 0.0 when max==min
and MSE==0 (constant signal). Returns infinity if max==min but
MSE>0 to flag codec bugs during selection.
```

## Verification Sequence

Before every commit, run this sequence in order. Do not skip steps.

1. `cargo fmt --check` — if it fails, run `cargo fmt` and re-check.
2. `cargo clippy --workspace --all-targets -- -D warnings` — fix all warnings.
3. `cargo test --workspace` — all tests must pass.
4. If the change involves a feature flag: also run `cargo test --workspace --features rayon`.
5. Only after all checks pass: stage and commit.

## Code Standards

- **No `.unwrap()` or `.expect()` in non-test code.** Every fallible operation returns `Result`. Use `?` for propagation.
- **No `panic!()` in library code.** Panics are only acceptable in tests and `unreachable!()` branches with a comment explaining why.
- **No `#![allow(dead_code)]` or blanket lint suppression.** If code is unused, remove it.
- **All public functions and types must have doc comments.** At minimum: one sentence describing what it does, and `# Errors` section listing when it returns `Err`.
- **All public functions returning `Result` must be `#[must_use]`.** (The thiserror `Error` type itself should be `#[must_use]`.)
- **Prefer `&[T]` over `Vec<T>` in function signatures** when the function doesn't need ownership.
- **No `usize` in wire format code.** Only fixed-width integers (`u8`, `u16`, `u32`, `u64`, `i64`, `f32`, `f64`).
- **Little-endian for all wire format integers.** Use `to_le_bytes()` / `from_le_bytes()`.

## Testing Requirements

- **Every new public function needs at least one unit test.** More for complex logic.
- **Every codec must have a roundtrip test:** `decompress(compress(data)) ≈ data` within error bound.
- **Every decode path must have a corrupt-input test:** feed truncated/garbage bytes, verify `Err` is returned (never panic).
- **Wire format tests use hand-crafted byte vectors** committed as constants, not generated at test time.
- **Benchmarks:** when changing codec hot paths, add or update a Criterion benchmark and include before/after numbers in the commit message body.

## File Organisation (PLAN.md is the source of truth)

Follow the directory layout defined in `PLAN.md` Phase 1.1. Do not create files outside that structure without updating the plan first.

- `atsc/src/error.rs` — error types
- `atsc/src/codec/` — codec trait + implementations
- `atsc/src/format/` — wire format encode/decode
- `atsc/src/optimizer/` — chunking + codec selection
- `atsc/src/vsri/` — timestamp compression
- `atsc/src/metrics.rs` — NRMSE
- `atsc-cli/src/main.rs` — CLI

## Execution Order

Follow the step order in `PLAN.md`. Do not jump ahead. Each step should be one or more commits that leave the workspace in a compiling, tested state. Never commit code that doesn't compile.

## What Not To Do

- Do not add dependencies without checking them against the target list in `PLAN.md` Phase 5.2.
- Do not modify `PLAN.md` without explicit approval.
- Do not introduce `unsafe` code.
- Do not add CLI features before the library API is complete (Phase 4.1 before 4.2).
- Do not optimise before the correct implementation exists. Correctness first, then benchmarks, then optimisation.
- Do not commit planning/complaint documents (for example `PLAN-*.md`, `COMPLAIN*.md`).
- Existing tracked `PLAN.md` is grandfathered for historical context, but agents must not add new plan/complaint files to the repository.
