# Decompression-First Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make ATSC's existing BRO v1 format safe and fast to decode from a reusable Rust library, then expose that boundary through a production-oriented CLI without materially increasing compressed output.

**Architecture:** Preserve the BRO v1 wire layout and existing convenience APIs while adding fallible bounded-encode and parse/decode paths. Bounded encoding validates full-frame error before transactionally committing a frame. A caller-owned `Decoder` holds reusable FFT plans and scratch buffers and can decode full streams, individual frames, or sample ranges into caller-owned output. The CLI becomes a thin adapter over this library API and adds inspect/verify commands while preserving the current invocation syntax.

**Tech Stack:** Rust 1.81, bincode 2.0.0-rc.3 for frozen BRO v1 compatibility, rustfft 6.x, Criterion, Clap 4, thiserror 2.

## Global Constraints

- Do not change the BRO v1 header, bincode configuration, codec enum order, codec payload layout, or existing golden output bytes.
- Existing valid BRO v1 fixtures must remain byte-for-byte identical when recompressed.
- Compression may become slower; decompression must show a statistically meaningful improvement on at least the FFT and multi-frame benchmarks, with no statistically meaningful regression in another codec.
- The 131,072-sample FFT correctness fix may change newly compressed large streams, but compressed size may not regress by more than 1% on any checked corpus case. Stop and reassess if that gate fails.
- Lossless codecs remain bit-exact. Bounded lossy output is committed only when its reported full-frame error is finite and within the requested bound; changing error-metric semantics is out of scope for this branch.
- Existing `atsc [OPTIONS] <INPUT>` and `atsc -u <INPUT>` commands remain supported.
- New public encode and decode APIs return typed errors. Existing panic-based APIs remain only as compatibility wrappers and are marked deprecated where practical.
- The core decode path remains synchronous and does not add an async runtime.
- No OpenSearch-specific adapter and no BRO v2 format are added on this branch.
- Follow TDD: add each behavioral test first, run it to observe the expected failure, then implement the smallest passing change.
- Create one focused commit after each task passes its specified checks.

## File Structure

- `atsc/src/error.rs`: public encode/decode error and decode-limit types.
- `atsc/src/decoder.rs`: reusable decoder context, scratch ownership, and full/frame/range APIs.
- `atsc/src/data.rs`: transactional bounded frame insertion, BRO v1 container parsing, stream metadata, and compatibility wrappers.
- `atsc/src/header.rs`: fallible nine-byte header parsing and version checks.
- `atsc/src/frame/mod.rs`: transactional bounded compression, Auto selection, frame metadata/getters, and fallible decode dispatch.
- `atsc/src/compressor/*.rs`: fallible codec payload parsing and decode-into implementations.
- `atsc/src/optimizer/mod.rs`: retains the 131,072-sample maximum frame size.
- `atsc/src/cli.rs`: CLI argument model and command execution.
- `atsc/src/main.rs`: logging, parsing, error-to-exit-code boundary only.
- `atsc/tests/v1_wire_compat.rs`: frozen v1 byte fixtures and round-trip checks.
- `atsc/tests/encode_api.rs`: bounded-error enforcement, Auto fallback, and transactional stream insertion.
- `atsc/tests/decode_api.rs`: malformed input, limits, metadata, frame/range decode behavior.
- `atsc/tests/cli_commands.rs`: new command behavior plus legacy compatibility.
- `atsc/benches/decompression_bench.rs`: codec and full-stream decompression benchmarks.
- `docs/usage.md` and `README.md`: stable CLI and library examples.
- `.github/workflows/build_and_test.yaml`: current runner/actions and explicit workspace checks.

---

### Task 1: Freeze BRO v1 Bytes and Establish Performance Baselines

**Files:**
- Create temporarily, then delete: `atsc/examples/generate_v1_fixtures.rs`
- Create: `atsc/tests/fixtures/v1/{constant,noop,rle,fft,polynomial,idw}.bro`
- Create: `atsc/tests/v1_wire_compat.rs`
- Create: `atsc/benches/decompression_bench.rs`
- Modify: `atsc/Cargo.toml`

**Interfaces:**
- Consumes: existing `CompressedStream`, `Compressor`, and `OptimizerPlan`.
- Produces: immutable BRO v1 fixture bytes and Criterion benchmark names used by all later tasks.

- [ ] **Step 1: Add a deterministic fixture generator before production changes**

Create `atsc/examples/generate_v1_fixtures.rs`:

```rust
use atsc::{compressor::Compressor, data::CompressedStream};
use std::{fs, path::PathBuf};

fn write(name: &str, compressor: Compressor, samples: &[f64]) {
    let mut stream = CompressedStream::new();
    stream.compress_chunk_with(samples, compressor);
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v1");
    fs::write(directory.join(format!("{name}.bro")), stream.to_bytes()).unwrap();
}

fn main() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v1");
    fs::create_dir_all(directory).unwrap();
    write("constant", Compressor::Constant, &[7.0; 16]);
    write("noop", Compressor::Noop, &[1.0, -2.0, 3.0, 4.0, 5.0]);
    write("rle", Compressor::RLE, &[1.0, 1.0, 2.0, 2.0, 1.0, 1.0]);
    let wave: Vec<f64> = (0..256).map(|i| ((i as f64) / 8.0).sin()).collect();
    write("fft", Compressor::FFT, &wave);
    write("polynomial", Compressor::Polynomial, &wave);
    write("idw", Compressor::Idw, &wave);
}
```

- [ ] **Step 2: Generate and freeze fixtures**

Run:

```bash
cargo run -p atsc --example generate_v1_fixtures
rm atsc/examples/generate_v1_fixtures.rs
```

Expected: six non-empty files under `atsc/tests/fixtures/v1/`; the temporary generator is absent.

- [ ] **Step 3: Add compatibility tests**

Create `atsc/tests/v1_wire_compat.rs` with one test per codec. Each test must recompress the same input and compare it to `include_bytes!("fixtures/v1/<codec>.bro")`. For Constant, Noop, and RLE, parse the frozen fixture and compare its decompressed output to the source samples with exact equality. For FFT, Polynomial, and IDW, decode both the frozen fixture and the newly generated bytes, assert that the two decoded vectors are exactly equal, assert that each decoded vector's length equals the source sample length, and assert that every decoded value is finite. Do not compare lossy decoded output to the source with `calculate_error` in Task 1; the current source-relative MAPE is known broken and changing error-metric semantics is out of scope.

The shared helper signatures are:

```rust
fn encode(compressor: Compressor, samples: &[f64]) -> Vec<u8>;
fn assert_lossless_fixture(name: &str, compressor: Compressor, samples: &[f64], fixture: &[u8]);
fn assert_lossy_fixture(name: &str, compressor: Compressor, samples: &[f64], fixture: &[u8]);
```

- [ ] **Step 4: Run compatibility tests**

Run:

```bash
cargo test -p atsc --test v1_wire_compat
```

Expected: six tests pass against untouched production code.

- [ ] **Step 5: Add a decompression-focused Criterion harness**

Create `atsc/benches/decompression_bench.rs` with benchmark groups:

```text
codec/constant/131072
codec/noop/131072
codec/rle/131072
codec/fft/8192
codec/polynomial/8192
codec/idw/2048
stream/mixed/6_frames
stream/fft/2x65536
```

Compression happens once before `b.iter`. Every iteration only calls the current decode API and passes the result to `black_box`. Generate deterministic constant, stepped, and sine-wave inputs in code; do not read files inside benchmark iterations.

Register:

```toml
[[bench]]
name = "decompression_bench"
harness = false
```

- [ ] **Step 6: Record the baseline**

Run:

```bash
cargo bench -p atsc --bench decompression_bench -- --save-baseline before-modernization
```

Expected: Criterion records all eight benchmark IDs without panics.

- [ ] **Step 7: Commit the guardrails**

```bash
git add atsc/Cargo.toml atsc/tests/fixtures/v1 atsc/tests/v1_wire_compat.rs atsc/benches/decompression_bench.rs
git commit -m "test: freeze BRO v1 and benchmark decoding"
```

---

### Task 2: Add Fallible, Bounded BRO v1 Parsing and Codec Decode

**Files:**
- Create: `atsc/src/error.rs`
- Create: `atsc/tests/decode_api.rs`
- Modify: `atsc/src/lib.rs`
- Modify: `atsc/src/header.rs`
- Modify: `atsc/src/data.rs`
- Modify: `atsc/src/frame/mod.rs`
- Modify: `atsc/src/compressor/mod.rs`
- Modify: `atsc/src/compressor/{constant,noop,rle,fft,polynomial}.rs`

**Interfaces:**
- Produces:

```rust
pub struct DecodeLimits {
    pub max_input_bytes: usize,
    pub max_frames: usize,
    pub max_samples: usize,
}

impl Default for DecodeLimits;

pub enum DecodeError;

impl CompressorHeader {
    pub fn try_from_slice(data: &[u8]) -> Result<Self, DecodeError>;
}

impl CompressedStream {
    pub fn try_from_bytes(data: &[u8]) -> Result<Self, DecodeError>;
    pub fn try_from_bytes_with_limits(
        data: &[u8],
        limits: DecodeLimits,
    ) -> Result<Self, DecodeError>;
    pub fn try_decompress(&self) -> Result<Vec<f64>, DecodeError>;
}

impl CompressorFrame {
    pub fn try_decompress(&self) -> Result<Vec<f64>, DecodeError>;
}

impl Compressor {
    pub fn try_decompress(
        &self,
        samples: usize,
        data: &[u8],
    ) -> Result<Vec<f64>, DecodeError>;
}
```

- [ ] **Step 1: Write malformed-container tests first**

In `atsc/tests/decode_api.rs`, add tests proving:

```rust
assert!(matches!(
    CompressedStream::try_from_bytes(&[]),
    Err(DecodeError::TruncatedHeader { actual: 0 })
));

assert!(matches!(
    CompressedStream::try_from_bytes(b"NOT-BRRO!"),
    Err(DecodeError::InvalidMagic { .. })
));

assert!(matches!(
    CompressedStream::try_from_bytes(&future_version_fixture),
    Err(DecodeError::UnsupportedVersion { found: 2, supported: 1 })
));

assert!(matches!(
    CompressedStream::try_from_bytes_with_limits(
        include_bytes!("fixtures/v1/constant.bro"),
        DecodeLimits { max_input_bytes: 8, ..DecodeLimits::default() },
    ),
    Err(DecodeError::InputLimitExceeded { .. })
));
```

Also mutate the fixture's ninth byte to create a header/body frame-count mismatch and truncate one byte from each codec fixture to verify a typed decode error rather than a panic.

- [ ] **Step 2: Run tests and observe failure**

Run:

```bash
cargo test -p atsc --test decode_api
```

Expected: compile failure because `DecodeError`, `DecodeLimits`, and `try_from_bytes` do not exist.

- [ ] **Step 3: Add typed errors and limits**

Create `atsc/src/error.rs` using `thiserror::Error` with these variants:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("BRO header requires 9 bytes, got {actual}")]
    TruncatedHeader { actual: usize },
    #[error("invalid BRO magic bytes: {found:?}")]
    InvalidMagic { found: [u8; 4] },
    #[error("BRO version {found} is newer than supported version {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },
    #[error("input is {actual} bytes, above configured limit {limit}")]
    InputLimitExceeded { actual: usize, limit: usize },
    #[error("stream contains {actual} frames, above configured limit {limit}")]
    FrameLimitExceeded { actual: usize, limit: usize },
    #[error("stream contains {actual} samples, above configured limit {limit}")]
    SampleLimitExceeded { actual: usize, limit: usize },
    #[error("header declares {header} frames but body contains {body}")]
    FrameCountMismatch { header: usize, body: usize },
    #[error("trailing bytes after BRO body: {remaining}")]
    TrailingBytes { remaining: usize },
    #[error("failed to decode {context}: {source}")]
    Bincode {
        context: &'static str,
        #[source]
        source: bincode::error::DecodeError,
    },
    #[error("invalid {codec:?} frame: {reason}")]
    InvalidFrame { codec: Compressor, reason: String },
    #[error("frame index {index} is outside stream frame count {frames}")]
    FrameOutOfBounds { index: usize, frames: usize },
    #[error("sample range {start}..{end} is outside stream length {samples}")]
    RangeOutOfBounds { start: usize, end: usize, samples: usize },
}

#[derive(Debug, Clone, Copy)]
pub struct DecodeLimits {
    pub max_input_bytes: usize,
    pub max_frames: usize,
    pub max_samples: usize,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 256 * 1024 * 1024,
            max_frames: u8::MAX as usize,
            max_samples: 131_072 * u8::MAX as usize,
        }
    }
}
```

Export `pub mod error;` from `lib.rs`.

- [ ] **Step 4: Make header and container parsing fallible**

Implement `CompressorHeader::try_from_slice` with explicit length, magic, and version checks. Change `get_frame_count` to take `&self`.

Implement `CompressedStream::try_from_bytes_with_limits` in this order:

1. Reject input over `max_input_bytes`.
2. Parse exactly the first nine bytes with `try_from_slice`.
3. Decode the frame vector with the unchanged `BinConfig::get()`.
4. Reject unconsumed trailing bytes.
5. Validate header/body frame count.
6. Validate frame and checked-sum sample limits.

`try_from_bytes` delegates with `DecodeLimits::default()`. Existing `from_bytes` delegates to `try_from_bytes(...).expect(...)`.

- [ ] **Step 5: Make every codec payload decode fallible**

For Constant, Noop, RLE, FFT, and Polynomial, add:

```rust
pub fn try_decompress(data: &[u8]) -> Result<Self, DecodeError>
```

Decode with the unchanged bincode config, reject trailing bytes, and map the bincode error with a codec-specific context string. Validate payload-local invariants:

- Constant ID is 30.
- Noop ID is 250.
- RLE ID is 60.
- FFT ID is 15.
- Polynomial `point_step != 0` and data points are non-empty.
- `Compressor::Auto` is rejected as an invalid stored frame instead of reaching `todo!()`.

Keep current `decompress` methods as wrappers around `try_decompress(...).expect(...)`.

- [ ] **Step 6: Wire fallible frame and stream decompression**

Implement `Compressor::try_decompress`, `CompressorFrame::try_decompress`, and `CompressedStream::try_decompress`. At this layer, validate invariants that need frame context:

- Noop decoded value count equals `sample_count`.
- RLE run starts include zero and are less than `sample_count`.
- Every FFT frequency position is less than the reconstructed FFT length.
- Polynomial payload type matches `Compressor::Polynomial` or `Compressor::Idw`.

Existing infallible methods remain wrappers.

- [ ] **Step 7: Run focused and compatibility tests**

Run:

```bash
cargo test -p atsc --test decode_api
cargo test -p atsc --test v1_wire_compat
cargo test -p atsc --lib
```

Expected: all pass; fixture bytes remain identical.

- [ ] **Step 8: Commit safe parsing**

```bash
git add atsc/src atsc/tests/decode_api.rs
git commit -m "feat: add bounded fallible BRO decoding"
```

---

### Task 3: Add Reusable Full, Frame, and Range Decoder APIs

**Files:**
- Create: `atsc/src/decoder.rs`
- Modify: `atsc/src/lib.rs`
- Modify: `atsc/src/data.rs`
- Modify: `atsc/src/frame/mod.rs`
- Modify: `atsc/tests/decode_api.rs`
- Modify: `atsc/benches/decompression_bench.rs`

**Interfaces:**
- Consumes: Task 2 fallible parsing and codec dispatch.
- Produces:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameInfo {
    pub index: usize,
    pub sample_offset: usize,
    pub sample_count: usize,
    pub compressor: Compressor,
    pub payload_bytes: usize,
}

pub struct Decoder;

impl Decoder {
    pub fn new() -> Self;
    pub fn decode(&mut self, stream: &CompressedStream) -> Result<Vec<f64>, DecodeError>;
    pub fn decode_into(
        &mut self,
        stream: &CompressedStream,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError>;
    pub fn decode_frame_into(
        &mut self,
        stream: &CompressedStream,
        frame_index: usize,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError>;
    pub fn decode_range(
        &mut self,
        stream: &CompressedStream,
        range: Range<usize>,
    ) -> Result<Vec<f64>, DecodeError>;
    pub fn decode_range_into(
        &mut self,
        stream: &CompressedStream,
        range: Range<usize>,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError>;
}

impl CompressedStream {
    pub fn frame_count(&self) -> usize;
    pub fn sample_count(&self) -> usize;
    pub fn frame_info(&self) -> impl ExactSizeIterator<Item = FrameInfo> + '_;
}
```

- [ ] **Step 1: Write decoder behavior tests**

Add tests for:

- `decode_into` appends to existing output and returns only the number appended.
- Reusing one `Decoder` for two streams produces correct independent output.
- `decode_frame_into` decodes only the requested frame.
- A range spanning the end of frame 0 and start of frame 1 equals the same slice from full decode.
- Empty `n..n` ranges succeed with zero output.
- Reversed and out-of-bounds ranges return `RangeOutOfBounds`.
- `frame_info` offsets are monotonic and final offset plus count equals stream `sample_count`.

- [ ] **Step 2: Run tests and observe failure**

Run:

```bash
cargo test -p atsc --test decode_api
```

Expected: compile failure because `Decoder`, `FrameInfo`, and metadata methods do not exist.

- [ ] **Step 3: Implement metadata and a minimal reusable decoder**

Create `decoder.rs`. Initially, `decode_into` may call `frame.try_decompress()` and `output.extend(frame_values)`; reserve `stream.sample_count()` before looping.

For `decode_range_into`, skip non-intersecting frames. Decode an intersecting frame into the decoder's private `frame_output: Vec<f64>`, then append only the local overlap:

```rust
let local_start = range.start.saturating_sub(frame_start);
let local_end = (range.end.min(frame_end)) - frame_start;
output.extend_from_slice(&self.frame_output[local_start..local_end]);
```

This task establishes behavior; Task 4 removes per-frame hot-path allocations.

- [ ] **Step 4: Make compatibility methods use Decoder**

`CompressedStream::try_decompress` constructs a default decoder and calls `decode`. `CompressedStream::decompress` remains the infallible wrapper.

- [ ] **Step 5: Extend benchmarks**

For every existing benchmark, add a `decoder/reused/...` function that constructs `Decoder` outside `b.iter`, clears one output vector inside the iteration, and calls `decode_into`.

- [ ] **Step 6: Verify behavior**

Run:

```bash
cargo test -p atsc --test decode_api
cargo test -p atsc --test v1_wire_compat
cargo test -p atsc --all-targets
```

Expected: all pass.

- [ ] **Step 7: Commit the public decoder API**

```bash
git add atsc/src atsc/tests/decode_api.rs atsc/benches/decompression_bench.rs
git commit -m "feat: add reusable frame and range decoder"
```

---

### Task 4: Remove Decode Hot-Path Allocations and Reuse FFT Plans

**Files:**
- Modify: `atsc/src/decoder.rs`
- Modify: `atsc/src/frame/mod.rs`
- Modify: `atsc/src/compressor/mod.rs`
- Modify: `atsc/src/compressor/{constant,noop,rle,fft,polynomial}.rs`
- Modify: `atsc/tests/decode_api.rs`
- Modify: `atsc/benches/decompression_bench.rs`

**Interfaces:**
- Preserves all Task 3 public signatures.
- Adds crate-private codec method:

```rust
fn try_decompress_into(
    &self,
    samples: usize,
    payload: &[u8],
    decoder: &mut Decoder,
    output: &mut Vec<f64>,
) -> Result<usize, DecodeError>;
```

- [ ] **Step 1: Add allocation-reuse regression tests**

Expose no scratch fields publicly. Add `#[cfg(test)]` methods on `Decoder` returning FFT and RLE scratch capacities, then test that:

1. Decode once.
2. Record capacity.
3. Decode the same stream again.
4. Capacity does not grow on the second decode.

Also test that `decode_into` preserves an existing prefix and appends exactly `sample_count` values for every codec fixture.

- [ ] **Step 2: Run tests and observe failure**

Run:

```bash
cargo test -p atsc --test decode_api
```

Expected: compile failure because scratch-capacity test hooks and direct decode-into do not exist.

- [ ] **Step 3: Give Decoder reusable state**

Use only standard-library synchronization/state and existing dependencies:

```rust
pub struct Decoder {
    fft_planner: FftPlanner<f32>,
    fft_buffer: Vec<Complex<f32>>,
    frame_output: Vec<f64>,
    rle_runs: Vec<(usize, f64)>,
}
```

One decoder is intended for one worker/thread and is reused across calls; no global lock or async runtime is introduced.

- [ ] **Step 4: Implement codec decode-into paths**

- Constant: reserve and `resize` the output tail with the constant.
- Noop: decode once, validate length, and `extend` converted values without cloning its decoded vector.
- RLE: clear/reuse `rle_runs`, flatten boundaries, sort them, resize the output once, and fill `output[base + start..base + end]` with `slice.fill(value)`.
- Polynomial: push directly to output. Retain current interpolation and rounding behavior exactly.
- IDW: push directly to output; avoid cloning `data_points` if the dependency API permits borrowing. If it requires ownership, preserve the clone and record this as follow-up.
- FFT: clear/resize `fft_buffer`, populate mirrored frequencies in place, call `fft_planner.plan_fft_inverse(len)`, process in place, and append normalized real values directly to output.

Existing codec `to_data` and `*_to_data` functions delegate to a fresh `Decoder` and remain byte/output compatible.

- [ ] **Step 5: Run correctness tests**

Run:

```bash
cargo test -p atsc --test decode_api
cargo test -p atsc --test v1_wire_compat
cargo test -p atsc --lib
```

Expected: all pass with exact lossless and existing lossy outputs.

- [ ] **Step 6: Compare Criterion baselines**

Run:

```bash
cargo bench -p atsc --bench decompression_bench -- --baseline before-modernization
```

Expected: reused FFT and mixed-stream paths improve without a statistically meaningful regression in another codec. If this gate fails, profile and fix before committing.

- [ ] **Step 7: Commit hot-path improvements**

```bash
git add atsc/src atsc/tests/decode_api.rs atsc/benches/decompression_bench.rs
git commit -m "perf: reuse decoder plans and output buffers"
```

---

### Task 5: Enforce v1 FFT and Bounded Encode Safety

**Files:**
- Modify: `atsc/src/error.rs`
- Modify: `atsc/src/compressor/mod.rs`
- Modify: `atsc/src/compressor/constant.rs`
- Modify: `atsc/src/compressor/fft.rs`
- Modify: `atsc/src/frame/mod.rs`
- Modify: `atsc/src/data.rs`
- Modify: `atsc/tests/decode_api.rs`
- Modify: `atsc/tests/integration_test.rs`
- Create: `atsc/tests/encode_api.rs`
- Create: `atsc/tests/compressed_size.rs`

**Interfaces:**
- Keeps the BRO v1 wire layout and `FrequencyPoint.pos: u16`.
- Retains the optimizer maximum frame size of 131,072 samples.
- Adds public `EncodeError::ErrorBoundNotMet { codec, requested, actual }` and
  `EncodeError::InvalidCompressionSpeed`.
- Adds `CompressorFrame::try_compress_bounded`,
  `CompressorFrame::try_compress_best`, and
  `CompressedStream::try_compress_chunk_bounded_with`.
- Existing infallible bounded methods delegate to the fallible methods and
  panic clearly without committing a failed frame.
- Forced Constant reports actual reconstruction error for nonconstant input
  while preserving identical payload bytes; constant input reports exactly
  zero error.
- Does not change `DecodeError` semantics.

- [ ] **Step 1: Write failing encode-contract and FFT boundary tests**

Using a deterministic positive alternating 131,072-sample signal, test forced
FFT through the fallible stream API. It must either decode within the requested
bound or return typed `ErrorBoundNotMet` while leaving frame and sample counts
unchanged. Test the same input through Auto and require a committed frame whose
decoded error is within the bound.

Add transactional invalid-speed and infallible-wrapper tests. Before the
implementation these tests fail because the typed error and fallible APIs do
not exist.

Add forced Constant regressions: `[1.0, 2.0]` at `0.03` returns typed
`ErrorBoundNotMet` without changing body or header frame counts, while a truly
constant vector succeeds with exact zero error and unchanged bytes/output.
Every transaction-failure assertion checks `header.get_frame_count()` as well
as body-derived frame and sample counts.

For `fft_trim`, explicitly prove position 65,535 is retained, position 65,536
is excluded without aliasing, and `max_freq == usize::MAX` neither overallocates
nor over-pops. Characterize repeatable equal-magnitude selection on Rust 1.81
without adding a positional tie-break that would change frozen fixture bytes.

- [ ] **Step 2: Add the typed transactional encode boundary**

Add public `EncodeError` without changing `DecodeError`. Forced bounded
compression must call `get_compress_bounded_results` and reject a non-finite
reported error or one above the requested bound before assigning frame data.

Make `constant_compressor` calculate error from the actual encoded/decoded
constant reconstruction for nonconstant input. Preserve its existing payload
bytes and return exactly `0.0` when every source sample is constant.

Implement fallible frame and stream methods. Close/push the frame and increment
the header count only after compression succeeds. Make every existing
infallible bounded method an `expect`-based compatibility wrapper around its
fallible equivalent.

Update the legacy CLI integration characterization: a forced FFT input whose
reported error is `0.04025997998108369` against the default `0.03` bound must
exit unsuccessfully with the clear compatibility-wrapper panic and must not
create a `.bro` output.

Retain Polynomial and IDW multi-file directory integration coverage. Their old
directory setup included `uptime.wbro`, whose zero samples make source-relative
MAPE undefined; use two bounded-valid `memory_used.wbro` copies so the test
continues to cover directory traversal and both output files without bypassing
the encode contract.

- [ ] **Step 3: Validate Auto on full-frame results**

Validate the compression-speed index before indexing its lookup table. Sampling
may rank candidates, but it cannot authorize a full frame. Evaluate every
eligible codec on the full data and choose the smallest payload whose reported
error is finite and within the requested bound.

Use fixed codec order to break equal payload-size ties deterministically. RLE's
reported zero error is the lossless fallback. Assign the chosen compressor,
payload, and sample count only after a valid result exists.

- [ ] **Step 4: Bound v1 FFT candidate storage**

Exclude every FFT candidate above `u16::MAX` before constructing
`FrequencyPoint`, then use `u16::try_from` for retained positions. Cap selected
vector capacity and heap pop count to the number of representable candidates.
Basic, hinted, and bounded FFT all share this path.

Keep the current magnitude-only heap ordering to preserve frozen bytes. Exact
fixture and repeatability tests pin the current toolchain behavior; equal-key
heap order across Rust versions remains a format-freeze concern.

- [ ] **Step 5: Correct and measure deterministic size gates**

The original zero-valued forced sine is invalid for bounded MAPE. Replace only
that corpus with 131,072 samples generated as
`10.0 + sin(index / 8.0)`. Keep forced FFT, maximum error `0.03`, compression
sample level `1`, and the 1% threshold unchanged.

At parent commit `f612040834053c52ba0b8f535d21494d11a9d4a5`, measure the
corrected corpus from a temporary detached worktree:

```bash
cargo test -p atsc --test task5_corrected_baseline -- --nocapture
```

Measured twice: **13,996 bytes**. Retain the existing baselines of 2,101 bytes
for stepped Auto and 1,568 bytes for slowly changing Auto. Assert each new
output is no more than `ceil(old_size * 1.01)` and print old/new bytes and
percentage in the assertion message.

- [ ] **Step 6: Run correctness, compatibility, size, and performance gates**

```bash
cargo test -p atsc --test encode_api
cargo test -p atsc --test compressed_size -- --nocapture
cargo test -p atsc --test decode_api
cargo test -p atsc --test integration_test
cargo test -p atsc --test e2e
cargo test -p atsc --test v1_wire_compat
cargo test --workspace --all-targets
cargo bench -p atsc --bench decompression_bench -- --baseline before-modernization
```

Expected: all tests and size gates pass, all six frozen fixtures remain exact,
and Task 4's FFT/mixed gains remain while Task 5 paths are neutral or better.

- [ ] **Step 7: Commit encode and FFT safety**

```bash
git add atsc/src atsc/tests/encode_api.rs atsc/tests/compressed_size.rs atsc/tests/integration_test.rs docs/superpowers/plans/2026-07-23-decompression-first-modernization.md
git commit -m "fix: enforce bounded compression contracts"
```

---

### Task 6: Expose the Safe Library Through Inspect and Verify CLI Commands

**Files:**
- Create: `atsc/src/cli.rs`
- Rewrite: `atsc/src/main.rs`
- Create: `atsc/tests/cli_commands.rs`
- Modify: `atsc/Cargo.toml`
- Modify: `docs/usage.md`
- Modify: `README.md`

**Interfaces:**
- Preserves legacy:

```text
atsc [OPTIONS] <INPUT>
atsc -u <INPUT>
```

- Adds:

```text
atsc inspect <INPUT> [--json]
atsc verify <INPUT>
atsc compress <INPUT> [-o <OUTPUT>] [existing compression options]
atsc decompress <INPUT> [-o <OUTPUT>]
```

- [ ] **Step 1: Write CLI tests first**

Using the existing `std::process::Command` pattern, test:

- `atsc inspect fixture.bro` exits 0 and prints version, frame count, sample count, codec, and payload bytes.
- `atsc inspect fixture.bro --json` exits 0 and emits parseable JSON with keys `version`, `frames`, `samples`, and `frame_details`.
- `atsc verify fixture.bro` exits 0 and prints `valid`.
- `atsc verify truncated.bro` exits non-zero and writes the typed error to stderr.
- `atsc compress input.wbro -o output.bro` and `atsc decompress output.bro -o restored.wbro` round trip.
- Existing `atsc input.wbro` and `atsc -u input.bro` still pass.

- [ ] **Step 2: Run tests and observe failure**

Run:

```bash
cargo test -p atsc --test cli_commands
```

Expected: failures because subcommands do not exist.

- [ ] **Step 3: Split command parsing from process entry**

`main.rs` becomes:

```rust
mod cli;

fn main() {
    env_logger::init();
    if let Err(error) = cli::run(cli::Args::parse()) {
        eprintln!("{error}");
        std::process::exit(error.exit_code());
    }
}
```

`main.rs` imports `clap::Parser`. `cli.rs` owns public `Args`, argument structs, file selection, output paths, and command execution. Add:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Decode(#[from] atsc::error::DecodeError),
    #[error("{0}")]
    Usage(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Decode(_) => 3,
            Self::Io(_) => 1,
        }
    }
}

pub fn run(args: Args) -> Result<(), CliError>;
```

Do not log sample data to stdout when `--json` is active.

- [ ] **Step 4: Add structured inspection**

Add direct dependencies on `serde` with `derive` and `serde_json`. Build inspection output from `CompressedStream::frame_info`; do not decompress for `inspect`. `verify` performs parse plus full decode so malformed codec payloads fail.

- [ ] **Step 5: Fix directory processing behavior**

Collect the initial directory entries before writing outputs, filter by the expected input extension, and invoke each entry once. A failed entry is reported once and is not retried after partial side effects.

- [ ] **Step 6: Run CLI and compatibility tests**

Run:

```bash
cargo test -p atsc --test cli_commands
cargo test -p atsc --test integration_test
cargo test -p atsc --test e2e
```

Expected: all pass.

- [ ] **Step 7: Update usage documentation**

Document explicit subcommands first and legacy syntax as compatibility mode. Include a Rust example:

```rust
let stream = CompressedStream::try_from_bytes(&bytes)?;
let mut decoder = Decoder::new();
let values = decoder.decode_range(&stream, 1_000..2_000)?;
```

- [ ] **Step 8: Commit CLI integration**

```bash
git add atsc/src atsc/tests/cli_commands.rs atsc/Cargo.toml Cargo.lock docs/usage.md README.md
git commit -m "feat: add safe inspect and verify CLI"
```

---

### Task 7: Modernize Build Guardrails and Run Final Verification

**Files:**
- Modify: `Cargo.toml`
- Modify: `atsc/Cargo.toml`
- Modify: `wavbrro/Cargo.toml`
- Modify: `vsri/Cargo.toml`
- Modify: `csv-compressor/Cargo.toml`
- Modify: `tools/Cargo.toml`
- Modify: `csv-compressor/src/main.rs`
- Modify: `tools/src/bin/wav2wbro.rs`
- Modify: `.github/workflows/build_and_test.yaml`
- Modify: `.github/workflows/release.yaml`
- Modify: `README.md`

**Interfaces:**
- No runtime API or wire-format changes.

- [ ] **Step 1: Remove existing warnings**

Remove unused Clap imports in the three binaries. Do not perform unrelated cleanup.

- [ ] **Step 2: Declare and tune supported builds**

Add `rust-version = "1.81"` to all five workspace package manifests. Add:

```toml
[profile.release]
opt-level = 3
codegen-units = 1
lto = "thin"
```

Do not set workspace-wide `panic = "abort"` because the library must remain unwind-safe for future FFI boundaries.

- [ ] **Step 3: Refresh CI**

Use `ubuntu-24.04`. Pin actions to these reviewed revisions:

```yaml
- uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
- uses: Swatinem/rust-cache@e18b497796c12c097a38f9edb9d0641fb99eee32 # v2
- uses: marvinpinto/action-automatic-releases@d68defdd11f9dcc7f52f35c1b7c236ee7513bcc1
```

Remove the protobuf installation and feature-powerset commands because this workspace has no protobuf crate or feature matrix. Run:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo bench -p atsc --bench decompression_bench --no-run
```

Keep release behavior otherwise unchanged.

- [ ] **Step 4: Run full local verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo bench -p atsc --bench decompression_bench -- --baseline before-modernization
git status --short
```

Expected:

- Formatting passes.
- Clippy reports zero warnings.
- All baseline and new tests pass.
- Criterion confirms the required decompression improvements and no meaningful codec regression.
- Only intended files are modified.

- [ ] **Step 5: Re-run compressed-byte verification**

Run:

```bash
cargo test -p atsc --test v1_wire_compat
cargo test -p atsc --test compressed_size -- --nocapture
```

Expected: all six v1 fixtures are byte-identical and all large-stream cases remain within the 1% size budget.

- [ ] **Step 6: Commit final guardrails**

```bash
git add Cargo.toml atsc/Cargo.toml wavbrro/Cargo.toml vsri/Cargo.toml csv-compressor/Cargo.toml tools/Cargo.toml csv-compressor/src/main.rs tools/src/bin/wav2wbro.rs .github/workflows README.md
git commit -m "ci: enforce safe and fast decoder builds"
```

## Plan Self-Review

- Spec coverage: byte stability, decompression performance, safe library API, frame/range decoding, CLI inspection/verification, CI, and documentation are covered.
- Task 1 characterization scope: lossless fixtures decode exactly to their source samples; lossy fixtures preserve exact wire bytes and exact existing decoded behavior, retain the source sample count, and contain only finite decoded values. The known-broken source-relative MAPE is not used as a quality gate.
- Deliberate exclusions: error-metric behavior changes, NaN-preservation format changes, BRO v2, JNI, and OpenSearch adapter work require separate designs because they can change compression semantics or deployment architecture.
- Placeholder scan: no TBD/TODO implementation steps remain. Task 5 intentionally records measured pre-change numeric sizes from its parent commit rather than inventing values.
- Type consistency: `EncodeError`, `DecodeError`, `DecodeLimits`, bounded stream/frame methods, `Decoder`, `FrameInfo`, and `CompressedStream` signatures are consistent across producing and consuming tasks.
