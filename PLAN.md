# ATSC v2 — Modernisation Plan

## Vision

Rebuild ATSC as a **first-class, embeddable time-series compression library** with a thin CLI on top. Keep the proven algorithms — FFT, polynomial/IDW, constant detection — but redesign every layer for correctness, performance, and clean integration.

**Design decisions (locked in):**

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Serialization | Manual byte-level encoding | Full control, zero serialization deps in hot path, optimisable per-byte |
| Workspace | `atsc` (lib) + `atsc-cli` (bin) | Minimal surface. VSRI merged in. WavBrro, CSV, tools dropped entirely. |
| FFT precision | f32 default, f64 opt-in | Best compression ratio. f64 behind feature/config for high-precision use cases. |
| FFT frequency encoding | Implicit ordering / bitpacked position indices | No `pos` field per-frequency. Most compact. Avoids u16/u32 width debates. |
| Parallelism | rayon behind feature flag | Chunks are independent; multi-core speedup with zero API cost when disabled. |
| Error metric | Single robust default (NRMSE) | Avoids MAPE zero-division, avoids SMAPE normalisation bugs. Clean, one metric. |
| Error bound behavior | Codecs fail hard (`Err`); optimizer catches and retries | Codecs return `Err(ErrorBoundNotMet)`. Optimizer collects best-effort from all codecs, picks lowest error if none meet bound. |
| NaN/Inf policy | Filter with log warning; reject flag available | Default: strip NaN/Inf, log warning, compress remaining. `CompressConfig::reject_nan_inf` flag for strict mode. |
| Backward compat | None | Clean break. New magic `ATSC`. Old `.bro` files are not readable. |
| Stream frame count | Footer (not header) | Enables streaming writes. Readers scan frames if footer absent (truncated stream). |
| Decode safety | Hardcoded resource limits | Compile-time caps prevent OOM from corrupt/malicious input. |
| VSRI invariants | Strict (4 rules) | First-frame rule, count match, strict monotonic, validated at encode/decode. |
| Integrity checks | None | Keep format simple. Integrity is the transport layer's job. |
| Codec evolution | New codec ID for breaking changes | No per-payload version byte. Codec IDs are cheap (u8 = 256 slots). |
| Gibbs sizing | Frozen heuristic | Algorithm documented in spec. No extra bytes in payload. New strategy = new codec ID. |

---

## Current-State Audit (what we're fixing)

### Algorithms worth keeping
- FFT frequency selection via BinaryHeap + iterative error-bounded loop
- Polynomial CatmullRom / IDW interpolation with bitdepth-aware encoding
- Constant detection (min == max)
- VSRI linear-equation model for constant-rate timestamp segments

### Critical bugs and limitations

| Area | Issue |
|------|-------|
| **Error handling** | ~40 `.unwrap()` / `panic!()` in serialization, deserialization, error metrics. Corrupt input crashes. |
| **Noop compressor** | Rounds f64 to i64 — silently destroys fractional data. Not "no-op" at all. |
| **Polynomial `point_step`** | `u8` overflows for large frames (131072 / 3 = 43690 wraps to garbage). |
| **VSRI `get_time`** | `y = y0 + m * x` instead of `y = y0 + m * (x - x0)`. Wrong for any segment where x0 != 0. |
| **VSRI types** | `i32` timestamps (2038 overflow). Magic `[i32; 4]` arrays instead of named struct. `index_file` baked into serialized data. |
| **Format** | `frame_size` uses `size_of_val` (stack size, not byte count). `frame_count` never incremented. No version field. |
| **Optimizer** | `get_compressor()` hardcoded to FFT. Error-bounded path is `todo!()`. |
| **Error metrics** | MAPE divides by zero on zero-valued data (NaN). SMAPE double-normalises. All panic on length mismatch. |
| **FFT precision** | f64 → f32 conversion silently loses precision; only logs on overflow. |
| **Dead weight** | `#![allow(dead_code)]` globally. ~6 unused deps. Empty writers module. bincode pinned to RC. |

---

## Target Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Consumers                          │
│  atsc-cli        Rust embed       future: C-FFI / WASM  │
└──────┬────────────────┬────────────────┬────────────────┘
       │                │                │
       ▼                ▼                ▼
┌─────────────────────────────────────────────────────────┐
│                    atsc  (library)                       │
│                                                         │
│  ┌───────────────┐  ┌────────────┐  ┌────────────────┐ │
│  │ Codec trait    │  │ Optimizer  │  │ Wire Format    │ │
│  │               │  │            │  │ (manual enc)   │ │
│  │  FftCodec     │  │  plan()    │  │  Header        │ │
│  │  PolyCodec    │  │  select()  │  │  Frame         │ │
│  │  ConstCodec   │  │  chunk()   │  │  Stream        │ │
│  │  NoopCodec    │  │            │  │  Footer        │ │
│  │              │  │            │  │                │ │
│  └───────────────┘  └────────────┘  └────────────────┘ │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────────┐ │
│  │ Error    │  │ Stats /  │  │ VSRI (timestamps)    │ │
│  │ types    │  │ NRMSE    │  │ i64, named segments  │ │
│  └──────────┘  └──────────┘  └───────────────────────┘ │
│                                                         │
│  feature: rayon                                       │
└─────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Library-first.** The library operates on `&[f64]` slices. No file I/O in the core. CLI is a thin consumer.
2. **`Result` everywhere.** No `.unwrap()` in non-test code. Corrupt data returns `Err`, never crashes.
3. **Trait-based codecs.** `Codec` trait makes adding algorithms a matter of implementing one trait.
4. **Manual wire format.** Every byte is explicit. Frames are self-describing (codec_id + payload_len) for streaming reads.
5. **Correct numerics.** Fix every overflow, precision loss, and metric bug.
6. **Feature-gated extras.** `rayon` for parallelism. Core has minimal deps.
7. **Hardened decode.** Compile-time resource limits prevent OOM/DoS from malicious input.
8. **Strict VSRI.** Four validated invariants ensure timestamp/value consistency.

---

## Wire Format v2

No backward compatibility. Clean design. New magic bytes (`ATSC`) ensure old tools fail fast on new files and vice versa.

### Stream layout

```
┌────────────────────────────────┐
│ Header (6 bytes)               │
├────────────────────────────────┤
│ Frame 0                        │
├────────────────────────────────┤
│ Frame 1                        │
├────────────────────────────────┤
│ ...                            │
├────────────────────────────────┤
│ Frame N-1                      │
├────────────────────────────────┤
│ Footer (8 bytes)               │
└────────────────────────────────┘
```

### Header (6 bytes, fixed)

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 4 | `[u8; 4]` | Magic: `ATSC` |
| 4 | 1 | `u8` | Format version (starts at `1`) |
| 5 | 1 | `u8` | Flags (bit 0: has inline VSRI as first frame, bits 1-7: reserved) |

No frame count in header — it lives in the footer.

### Frame (9 + payload_len bytes)

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 1 | `u8` | Codec ID |
| 1 | 4 | `u32` LE | Sample count |
| 5 | 4 | `u32` LE | Payload length in bytes |
| 9 | var | `[u8]` | Codec-specific payload |

A reader can skip any frame by reading 9 bytes then seeking `payload_len` forward — no need to understand the codec.

### Footer (8 bytes, fixed)

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 4 | `[u8; 4]` | Magic: `CSTA` (reverse of `ATSC` — unambiguous marker) |
| 4 | 4 | `u32` LE | Frame count |

If the footer is missing (truncated stream), readers scan frames from after the header until EOF. The footer is advisory — it's a fast-path, not a requirement.

### Codec IDs

| ID | Codec |
|----|-------|
| 0 | Noop (lossless raw f64) |
| 1 | Constant |
| 2 | FFT (f32 frequencies, bitpacked positions) |
| 3 | Reserved (deprecated: FFT f64 was removed from v2) |
| 4 | Polynomial (CatmullRom) |
| 5 | Reserved (deprecated: IDW was removed from v2) |
| 128 | VSRI (timestamps) |

### Decode resource limits (compile-time constants)

```rust
pub const MAX_FRAMES: u32 = 1_000_000;
pub const MAX_PAYLOAD_BYTES: u32 = 256 * 1024 * 1024;  // 256 MB per frame
pub const MAX_TOTAL_SAMPLES: u64 = 1_000_000_000;       // 1 billion samples
pub const MAX_VSRI_SEGMENTS: u32 = 10_000_000;
```

Decode functions check these before allocating. Returns `Error::ResourceLimitExceeded` on violation.

### VSRI invariants (enforced at encode and decode)

1. If header flag bit 0 is set, the **first frame** must have `codec_id = 128` (VSRI).
2. VSRI segment total sample count must equal the sum of all value frame `sample_count` fields.
3. Timestamps must be **strictly monotonically increasing** (no duplicates).
4. All invariant violations return `Error::VsriInvariantViolation` with a descriptive message.

### NaN/Inf input policy

- **Default behavior:** strip NaN and Inf values from input before compression. Log a warning with the count of removed values. The `sample_count` in the output reflects the *filtered* length.
- **Strict mode:** if `CompressConfig::reject_nan_inf = true`, return `Error::InvalidInput` immediately when NaN/Inf is detected.
- The library never silently changes data semantics — either it removes values with a logged warning, or it rejects outright.

### Codec-specific payloads

**Noop (ID 0):** raw little-endian f64 values. `payload_len = sample_count * 8`.

**Constant (ID 1):**

| Field | Size | Type |
|-------|------|------|
| value | 8 | `f64` LE |

`payload_len = 8`. Decompression: `vec![value; sample_count]`.

**FFT f32 (ID 2):**

| Field | Size | Type | Notes |
|-------|------|------|-------|
| min_value | 4 | `f32` LE | Clamping floor for reconstruction |
| max_value | 4 | `f32` LE | Clamping ceiling for reconstruction |
| freq_count | 2 | `u16` LE | Number of retained frequencies |
| position_index | variable | bitpacked | See below |
| freq_values | `freq_count * 8` | `[f32 LE, f32 LE]` | (real, imag) pairs |

**Position index encoding:** frequencies are stored sorted by magnitude (descending). Their original positions in the FFT output are encoded as a bitpacked index:

- Compute `bits_per_pos = ceil(log2(fft_len))` where `fft_len` is the (frozen-heuristic) padded length.
- Pack `freq_count` positions, each `bits_per_pos` bits wide, into a byte-aligned buffer.
- `position_index_len = ceil(freq_count * bits_per_pos / 8)` bytes.

This removes the per-frequency `pos` field entirely. For a 65536-sample FFT with 100 frequencies: `ceil(100 * 16 / 8) = 200` bytes for positions, vs. 200 bytes with u16 per-frequency — same for this case, but scales to any FFT length without type-width limits.

The `fft_len` needed for decoding is deterministic from `sample_count` via the frozen Gibbs heuristic (documented below).

`payload_len = 4 + 4 + 2 + position_index_len + freq_count * 8`.

**FFT f64 (ID 3):** deprecated / reserved. v2 does not implement this codec id.

**Frozen Gibbs heuristic (spec):** given `sample_count`:
- If `sample_count < 128`: `fft_len = sample_count` (no padding).
- Otherwise: `fft_len = next_smooth(sample_count)` where `next_smooth(n)` returns the smallest integer `>= n` of the form `2^a * 3^b`.
- Padding: `prefix_len = (fft_len - sample_count) / 2`, `suffix_len = fft_len - sample_count - prefix_len`.
- Prefix filled with `data[0]`; suffix filled with `data[last]`.

This algorithm is frozen for codec ID 2. Any change to the padding strategy requires allocating new codec IDs.

**Polynomial (ID 4):**

| Field | Size | Type |
|-------|------|------|
| min | 8 | `f64` LE |
| max | 8 | `f64` LE |
| point_step | 4 | `u32` LE |
| bitdepth | 1 | `u8` (0=u8, 1=i16, 2=i32, 3=f64) |
| point_count | 4 | `u32` LE |
| data_points | variable | encoded per bitdepth |

`point_step` is `u32` — no overflow.

**VSRI (ID 128):**

| Field | Size | Type |
|-------|------|------|
| min_ts | 8 | `i64` LE |
| max_ts | 8 | `i64` LE |
| segment_count | 4 | `u32` LE |
| segments | `segment_count * 28` | `[Segment]` |

Each `Segment`:

| Field | Size | Type |
|-------|------|------|
| rate | 8 | `i64` LE |
| x0 | 8 | `i64` LE |
| y0 | 8 | `i64` LE |
| count | 4 | `u32` LE |

28 bytes per segment. All i64 — no 2038 problem.

---

## Phase 1 — Foundation

### 1.1 Workspace restructure

**New workspace:**

```
fft-compression/
├── Cargo.toml          (workspace: members = ["atsc", "atsc-cli"])
├── atsc/
│   ├── Cargo.toml      (lib crate)
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       ├── codec/
│       │   ├── mod.rs      (Codec trait)
│       │   ├── fft.rs
│       │   ├── polynomial.rs
│       │   ├── constant.rs
│       │   └── noop.rs
│       ├── format/
│       │   ├── mod.rs
│       │   ├── header.rs
│       │   ├── frame.rs
│       │   ├── footer.rs
│       │   ├── stream.rs
│       │   └── limits.rs   (decode resource limits)
│       ├── optimizer/
│       │   ├── mod.rs
│       │   ├── stats.rs
│       │   └── chunker.rs
│       ├── vsri/
│       │   ├── mod.rs
│       │   └── segment.rs
│       └── metrics.rs      (NRMSE)
└── atsc-cli/
    ├── Cargo.toml      (bin crate, depends on atsc)
    └── src/
        └── main.rs
```

**Dropped entirely:** `wavbrro`, `csv-compressor`, `tools`, `lib_vsri` (merged into `atsc::vsri`).

The CLI accepts **raw binary f64 files** as input (simple: just `N * 8` bytes of little-endian f64). No WavBrro dependency. Existing `.wbro` test data can be converted once with a one-off script.

### 1.2 Error types

```rust
// atsc/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("data is empty")]
    EmptyData,

    #[error("input contains NaN or Inf values (strict mode enabled)")]
    InvalidInput,

    #[error("decode: unexpected end of input at offset {offset}, expected {expected} bytes")]
    UnexpectedEof { offset: usize, expected: usize },

    #[error("decode: invalid magic bytes")]
    InvalidMagic,

    #[error("decode: unsupported format version {0}")]
    UnsupportedVersion(u8),

    #[error("decode: unknown codec id {0}")]
    UnknownCodec(u8),

    #[error("decode: resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("encode: sample count {count} exceeds u32::MAX")]
    SampleCountOverflow { count: usize },

    #[error("error bound not satisfiable: best={best:.6}, target={target:.6}")]
    ErrorBoundNotMet { best: f64, target: f64 },

    #[error("precision overflow converting f64 to f32: {0}")]
    PrecisionOverflow(f64),

    #[error("vsri: timestamp {ts} is not monotonically increasing (max was {max})")]
    VsriOutOfOrder { ts: i64, max: i64 },

    #[error("vsri invariant violation: {0}")]
    VsriInvariantViolation(String),

    #[error("metrics: original and reconstructed lengths differ ({a} vs {b})")]
    LengthMismatch { a: usize, b: usize },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### 1.3 Codec trait

```rust
// atsc/src/codec/mod.rs
pub trait Codec: Send + Sync {
    fn id(&self) -> u8;
    fn name(&self) -> &'static str;

    /// Compress data. Returns Err(ErrorBoundNotMet) if max_error cannot be met
    /// within max_iterations. The optimizer catches this and selects the best
    /// available result across all codecs.
    fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame>;

    fn decompress(&self, payload: &[u8], sample_count: u32) -> Result<Vec<f64>>;
}

#[derive(Clone, Debug)]
pub struct CompressConfig {
    pub max_error: Option<f64>,
    pub max_iterations: u32,
    pub reject_nan_inf: bool,
}

impl Default for CompressConfig {
    fn default() -> Self {
        Self {
            max_error: Some(0.05),
            max_iterations: 22,
            reject_nan_inf: false,
        }
    }
}

pub struct CompressedFrame {
    pub codec_id: u8,
    pub sample_count: u32,
    pub payload: Vec<u8>,
    pub measured_error: f64,
}
```

### 1.4 Wire format implementation

Manual encode/decode in `atsc/src/format/`:

```rust
// Encode
pub fn encode_stream(header: &Header, frames: &[CompressedFrame]) -> Result<Vec<u8>>;

// Decode
pub fn decode_header(buf: &[u8]) -> Result<Header>;
pub fn decode_frame(buf: &[u8], offset: &mut usize) -> Result<CompressedFrame>;
pub fn decode_footer(buf: &[u8]) -> Result<Option<Footer>>;
pub fn decode_stream(buf: &[u8]) -> Result<(Header, Vec<CompressedFrame>)>;
```

All reads are bounds-checked, returning `Error::UnexpectedEof` on truncation. Little-endian throughout. No padding, no alignment requirements. No `usize` in the wire format — only fixed-width integers.

Decode enforces resource limits from `format::limits` before every allocation.

### 1.5 Feature flags

```toml
[features]
default = []
rayon = ["dep:rayon"]
```

---

## Phase 2 — Codecs (migrate + fix)

### 2.1 FFT

Port the algorithm from `atsc/src/compressor/fft.rs` into `atsc/src/codec/fft.rs`, implementing `Codec`.

Fixes:
- **Frequency positions:** bitpacked index encoding. No per-frequency `pos` field. Frequencies sorted by magnitude. Position index uses `ceil(log2(fft_len))` bits per position, byte-aligned.
- **f64_to_f32**: return `Error::PrecisionOverflow` on non-finite result.
- **Bounded loop**: use `(best_error - target).abs() < epsilon` instead of truncated integer comparison. Codec returns `Err(ErrorBoundNotMet { best, target })` if budget exhausted.
- **Dedup stats**: single `DataStats::new(data)` call, remove manual min/max loops.
- **Gibbs sizing**: frozen heuristic (documented in wire format spec above). No extra bytes in payload.
- **f64 mode**: removed (codec ID 3 is reserved / deprecated).
- **Manual payload encode/decode**: hand-written LE byte serialization with bitpacking for positions.

### 2.2 Polynomial / IDW

Port into `atsc/src/codec/polynomial.rs`, implementing `Codec`.

Fixes:
- **`point_step`**: `u32` (was `u8` — critical overflow bug).
- **Unify types**: single `InterpolationMethod` enum replaces `PolynomialType` + `Method`.
- **Bounded loop fallback**: return `Err(ErrorBoundNotMet { best, target })` when budget exhausted.
- **Manual payload encode/decode**: bitdepth-aware, hand-written.

### 2.3 Constant

Port into `atsc/src/codec/constant.rs`. Simplest codec.

- Payload: 8 bytes (one f64 LE).
- Remove unused constructor parameters.
- Error is always 0.0 for truly constant data.

### 2.4 Noop

Port into `atsc/src/codec/noop.rs`.

- **Make truly lossless**: store raw f64 LE bytes. No rounding. `payload_len = sample_count * 8`.
- Error is always 0.0.
- This is the "I don't want any loss" escape hatch and the fallback when all other codecs fail.

### 2.5 VSRI

Port `lib_vsri` into `atsc/src/vsri/`.

Fixes:
- **`get_time` bug**: `y = y0 + m * (x - x0)`.
- **Named struct**: `Segment { rate: i64, x0: i64, y0: i64, count: u32 }`.
- **i64 everywhere**: no 2038 problem.
- **Drop `index_file`**: not part of data; persistence is the caller's concern.
- **Drop `day_elapsed_seconds` / `start_day_ts` / `MAX_INDEX_SAMPLES`**: not VSRI concerns.
- **Strict monotonic**: `update_for_point` rejects `ts <= max_ts` (no duplicates).
- **Proper errors**: `Error::VsriOutOfOrder` and `Error::VsriInvariantViolation` with context.
- **Manual encode/decode**: `Segment` array to/from bytes.

### 2.6 Error metrics

New `atsc/src/metrics.rs`:

```rust
/// Normalised Root Mean Square Error.
/// Returns 0.0 when max == min and MSE == 0 (constant signal, perfect reconstruction).
/// Returns Err(LengthMismatch) if slices differ in length.
pub fn nrmse(original: &[f64], reconstructed: &[f64]) -> Result<f64>;
```

- NRMSE = `sqrt(MSE) / (max - min)`.
- **Constant signal special case:** when `max == min` and `MSE == 0`, return `0.0`. If `max == min` but `MSE > 0`, this indicates a codec bug — return `f64::INFINITY` (infinite relative error) so the optimizer never selects it.
- Single metric. No MAPE, SMAPE, MAE, MSE as public API.

---

## Phase 3 — Optimizer

### 3.1 Compressor selection

Replace the hardcoded `Compressor::FFT` with actual selection logic:

```rust
pub fn select_codec(
    data: &[f64],
    codecs: &[&dyn Codec],
    config: &CompressConfig,
) -> Result<CompressedFrame>
```

Strategy:
1. **Constant check**: if `min == max`, use Constant immediately (0 error, 8 bytes).
2. **Try all codecs** on the data (or a subsample if `compression_speed` allows). Each codec returns `Ok(frame)` or `Err(ErrorBoundNotMet { best, target })`.
3. From successful results, **pick smallest payload** that meets the error bound.
4. If all codecs return `Err(ErrorBoundNotMet)`: collect the `best` error from each, pick the codec with the lowest `best` error, re-run it with that budget, and return its frame. **Noop is always available as a last resort** (0 error, but no compression).

With `rayon` feature: trial compressions across codecs run in parallel.

### 3.2 Chunking

- Power-of-2 chunks for FFT (keep existing logic — it works).
- Min chunk: 512. Max chunk: 131072. Same as today.
- Regime-change detection deferred to a future iteration.

### 3.3 Optimizer plan

```rust
pub struct Plan {
    pub chunks: Vec<Range<usize>>,
}

impl Plan {
    pub fn new(data_len: usize) -> Self;
}
```

Simpler than `OptimizerPlan` — just computes chunk boundaries. Codec selection happens per-chunk at compression time, not at plan time.

---

## Phase 4 — Public API

### 4.1 Library API

```rust
// === One-shot ===

/// Compress f64 time-series data.
pub fn compress(data: &[f64], config: &CompressConfig) -> Result<Vec<u8>>;

/// Decompress back to f64.
pub fn decompress(bytes: &[u8]) -> Result<Vec<f64>>;

// === With timestamps ===

/// Compress values + timestamps together (VSRI frame prepended, flag set).
pub fn compress_with_timestamps(
    timestamps: &[i64],
    values: &[f64],
    config: &CompressConfig,
) -> Result<Vec<u8>>;

/// Decompress values + timestamps.
pub fn decompress_with_timestamps(bytes: &[u8]) -> Result<(Vec<i64>, Vec<f64>)>;

// === Streaming / frame-level ===

pub struct StreamWriter { ... }
impl StreamWriter {
    pub fn new(config: CompressConfig) -> Self;
    pub fn push(&mut self, data: &[f64]) -> Result<()>;
    pub fn finish(self) -> Result<Vec<u8>>;  // writes footer with frame count
}

pub struct StreamReader<'a> { ... }
impl<'a> StreamReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self>;
    pub fn header(&self) -> &Header;
    pub fn frame_count(&self) -> u32;  // from footer, or scanned
    pub fn frames(&self) -> FrameIter<'a>;
}

// === Inspect ===

pub fn inspect(bytes: &[u8]) -> Result<StreamInfo>;

pub struct StreamInfo {
    pub version: u8,
    pub frame_count: u32,
    pub total_samples: u64,
    pub has_vsri: bool,
    pub frames: Vec<FrameInfo>,
}

pub struct FrameInfo {
    pub codec_name: &'static str,
    pub codec_id: u8,
    pub sample_count: u32,
    pub payload_bytes: u32,
}
```

### 4.2 CLI (`atsc-cli`)

```
atsc compress <input> [-o output.atsc] [--codec auto|fft|poly|const|noop] [--error 5] [--strict]
atsc decompress <input.atsc> [-o output]
atsc inspect <input.atsc>
atsc bench <input>     # runs all codecs, prints table of ratio/error/time
```

- Input: raw binary f64 files (simple `N * 8` bytes LE). No WavBrro.
- Output: `.atsc` files (new format).
- `--strict`: enables `reject_nan_inf`.
- The CLI does file I/O; the library never does.
- `atsc inspect` prints header info, frame-by-frame codec/size/samples, and footer.

---

## Phase 5 — Quality

### 5.1 Testing

- **Roundtrip property tests** (`proptest`): for every codec, `decompress(compress(data))` is within error bound for random data.
- **Fuzz targets** (`cargo-fuzz`): feed random bytes to `decompress()`, `decode_stream()`, `decode_header()`. Must never panic, only return `Err`.
- **Wire format golden tests**: hand-craft byte sequences for each codec, verify decode produces expected output. Encode then decode, verify roundtrip. Commit golden vectors to the repo.
- **Resource limit tests**: craft payloads with `frame_count = u32::MAX`, `payload_len = u32::MAX`, etc. Verify `Error::ResourceLimitExceeded` is returned without allocating.
- **VSRI invariant tests**: verify that encode/decode rejects non-monotonic timestamps, mismatched sample counts, missing VSRI frame when flag is set.
- **Regression benchmarks** (Criterion): FFT compress/decompress at 256/1024/4096/65536 sizes. Polynomial same.

### 5.2 Dependencies (target)

```toml
# atsc/Cargo.toml
[dependencies]
thiserror = "2"
rustfft = "6.1"
splines = "4.3"
log = "0.4"
rayon = { version = "1", optional = true }

[dev-dependencies]
criterion = "0.5"
proptest = "1"
```

No bincode, no rkyv, no hound, no regex, no average, no median, no chrono, no tempfile, no csv, no wavbrro.

```toml
# atsc-cli/Cargo.toml
[dependencies]
atsc = { path = "../atsc" }
clap = { version = "4", features = ["derive"] }
env_logger = "0.11"
log = "0.4"
```

### 5.3 CI

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -D warnings` (no allows)
- `cargo test --workspace`
- `cargo test --workspace --features rayon`
- `cargo bench` (Criterion regression check)
- `cargo deny check` or `cargo audit`

### 5.4 Lint hygiene

- Remove `#![allow(dead_code)]`.
- Remove `#![allow(clippy::...)]` unless justified with a comment.
- `#[must_use]` on all `Result`-returning public functions.

---

## Execution Order

| Step | Description | Depends on |
|------|-------------|------------|
| 1 | Workspace restructure: create `atsc/` (lib) + `atsc-cli/` (bin), remove dropped crates | — |
| 2 | `error.rs`: define `Error` enum + `Result` alias | 1 |
| 3 | `format/limits.rs`: define decode resource limit constants | 2 |
| 4 | `codec/mod.rs`: define `Codec` trait, `CompressConfig`, `CompressedFrame` | 2 |
| 5 | `format/`: implement header + frame + footer encode/decode with limits | 2, 3, 4 |
| 6 | `metrics.rs`: implement NRMSE with constant-signal special case | 2 |
| 7 | `codec/constant.rs`: implement Constant codec | 4, 5 |
| 8 | `codec/noop.rs`: implement Noop codec (truly lossless) | 4, 5 |
| 9 | `codec/fft.rs`: port + fix FFT codec with bitpacked positions | 4, 5, 6 |
| 10 | `codec/polynomial.rs`: port + fix Polynomial/IDW codec | 4, 5, 6 |
| 11 | `vsri/`: port + fix VSRI with i64 segments + invariant validation | 2, 5 |
| 12 | `optimizer/`: implement chunker + codec selection with fail-hard + fallback | 4, 6, 7–10 |
| 13 | Public API: `compress()`, `decompress()`, `StreamWriter`, `StreamReader`, `inspect()` | 5, 11, 12 |
| 14 | `atsc-cli`: build CLI with subcommands | 13 |
| 15 | Property tests + fuzz targets + wire format golden tests + VSRI invariant tests | 13 |
| 16 | CI setup + lint cleanup + dependency audit | 14, 15 |
