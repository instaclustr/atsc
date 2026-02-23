# ATSC v2 — Modernisation Plan

## Vision

Rebuild ATSC as a **first-class, embeddable time-series compression library** with a thin CLI on top. Keep the proven algorithms — FFT, polynomial/IDW, constant detection — but redesign every layer for correctness, performance, and clean integration.

**Design decisions (locked in):**

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Serialization | Manual byte-level encoding | Full control, zero serialization deps in hot path, optimisable per-byte |
| Workspace | `atsc` (lib) + `atsc-cli` (bin) | Minimal surface. VSRI merged in. WavBrro, CSV, tools dropped. |
| FFT precision | f32 default, f64 opt-in | Best compression ratio. f64 behind feature/config for high-precision use cases. |
| Parallelism | rayon behind feature flag | Chunks are independent; multi-core speedup with zero API cost when disabled. |
| Error metric | Single robust default (NRMSE) | Avoids MAPE zero-division, avoids SMAPE normalisation bugs. Clean, one metric. |
| Backward compat | None | Clean break. No legacy decode paths. |
| Dropped | `csv-compressor`, `tools`, `wavbrro` | Out of scope for v2 core. Can be built as adapters later. |

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
│  │  FftCodec     │  │  plan()    │  │  Stream        │ │
│  │  PolyCodec    │  │  select()  │  │  Frame         │ │
│  │  ConstCodec   │  │  chunk()   │  │  Header        │ │
│  │  NoopCodec    │  │            │  │                │ │
│  │  (IdwCodec)   │  │            │  │                │ │
│  └───────────────┘  └────────────┘  └────────────────┘ │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────────┐ │
│  │ Error    │  │ Stats /  │  │ VSRI (timestamps)    │ │
│  │ types    │  │ NRMSE    │  │ i64, named segments  │ │
│  └──────────┘  └──────────┘  └───────────────────────┘ │
│                                                         │
│  feature: rayon  │  feature: fft-f64                    │
└─────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Library-first.** The library operates on `&[f64]` slices. No file I/O in the core. CLI is a thin consumer.
2. **`Result` everywhere.** No `.unwrap()` in non-test code. Corrupt data returns `Err`, never crashes.
3. **Trait-based codecs.** `Codec` trait makes adding algorithms a matter of implementing one trait.
4. **Manual wire format.** Every byte is explicit. Frames are self-describing (codec_id + payload_len) for streaming reads.
5. **Correct numerics.** Fix every overflow, precision loss, and metric bug.
6. **Feature-gated extras.** `rayon` for parallelism, `fft-f64` for double-precision frequencies. Core has minimal deps.

---

## Wire Format v2

No backward compatibility. Clean design.

### Stream layout

```
┌────────────────────────────────┐
│ Header (8 bytes)               │
├────────────────────────────────┤
│ Frame 0                        │
├────────────────────────────────┤
│ Frame 1                        │
├────────────────────────────────┤
│ ...                            │
├────────────────────────────────┤
│ Frame N-1                      │
└────────────────────────────────┘
```

### Header (8 bytes, fixed)

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 4 | `[u8; 4]` | Magic: `ATSC` |
| 4 | 1 | `u8` | Format version (starts at `1`) |
| 5 | 1 | `u8` | Flags (bit 0: has inline VSRI frame, bits 1-7: reserved) |
| 6 | 2 | `u16` LE | Frame count |

Magic changes from `BRRO` to `ATSC` — clean break, no ambiguity with v1 files.

### Frame (9 + payload_len bytes)

| Offset | Size | Type | Field |
|--------|------|------|-------|
| 0 | 1 | `u8` | Codec ID |
| 1 | 4 | `u32` LE | Sample count |
| 5 | 4 | `u32` LE | Payload length in bytes |
| 9 | var | `[u8]` | Codec-specific payload |

Codec IDs:

| ID | Codec |
|----|-------|
| 0 | Noop (lossless raw f64) |
| 1 | Constant |
| 2 | FFT (f32 frequencies) |
| 3 | FFT (f64 frequencies) |
| 4 | Polynomial (CatmullRom) |
| 5 | IDW |
| 128 | VSRI (timestamps) |

The frame header is always 9 bytes. A reader can skip any frame by reading 9 bytes then seeking `payload_len` forward — no need to understand the codec.

### Codec-specific payloads

**Noop (ID 0):** raw little-endian f64 values. `payload_len = sample_count * 8`.

**Constant (ID 1):**

| Field | Size | Type |
|-------|------|------|
| value | 8 | `f64` LE |

`payload_len = 8`. Decompression: `vec![value; sample_count]`.

**FFT f32 (ID 2):**

| Field | Size | Type |
|-------|------|------|
| min_value | 4 | `f32` LE |
| max_value | 4 | `f32` LE |
| freq_count | 2 | `u16` LE |
| frequencies | `freq_count * 8` | `[FreqPoint]` |

Each `FreqPoint` = `pos: u16 LE` + `real: f32 LE` (total per point: not needed, see below — actually let me reconsider).

Correction — each `FreqPoint`:

| Field | Size | Type |
|-------|------|------|
| pos | 2 | `u16` LE |
| real | 4 | `f32` LE |
| imag | 4 | `f32` LE |

10 bytes per frequency point. `payload_len = 4 + 4 + 2 + freq_count * 10`.

**FFT f64 (ID 3):** same layout but `f64` for real/imag (18 bytes per point, min/max are `f64`).

**Polynomial (ID 4) / IDW (ID 5):**

| Field | Size | Type |
|-------|------|------|
| min | 8 | `f64` LE |
| max | 8 | `f64` LE |
| point_step | 4 | `u32` LE |
| bitdepth | 1 | `u8` (0=u8, 1=i16, 2=i32, 3=f64) |
| point_count | 4 | `u32` LE |
| data_points | variable | encoded per bitdepth |

`point_step` is now `u32` — no overflow.

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
│       │   └── stream.rs
│       ├── optimizer/
│       │   ├── mod.rs
│       │   ├── stats.rs
│       │   └── chunker.rs
│       ├── vsri/
│       │   ├── mod.rs
│       │   └── segment.rs
│       └── metrics.rs      (NRMSE + helpers)
└── atsc-cli/
    ├── Cargo.toml      (bin crate, depends on atsc)
    └── src/
        └── main.rs
```

**Dropped crates:** `wavbrro`, `csv-compressor`, `tools`, `lib_vsri` (merged into `atsc::vsri`).

### 1.2 Error types

```rust
// atsc/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("data is empty")]
    EmptyData,

    #[error("decode: unexpected end of input at offset {offset}, expected {expected} bytes")]
    UnexpectedEof { offset: usize, expected: usize },

    #[error("decode: invalid magic bytes")]
    InvalidMagic,

    #[error("decode: unsupported format version {0}")]
    UnsupportedVersion(u8),

    #[error("decode: unknown codec id {0}")]
    UnknownCodec(u8),

    #[error("encode: sample count {count} exceeds u32::MAX")]
    SampleCountOverflow { count: usize },

    #[error("error bound not satisfiable: best={best:.6}, target={target:.6}")]
    ErrorBoundNotMet { best: f64, target: f64 },

    #[error("precision overflow converting f64 to f32: {0}")]
    PrecisionOverflow(f64),

    #[error("vsri: timestamp {ts} is not monotonically increasing (max was {max})")]
    VsriOutOfOrder { ts: i64, max: i64 },

    #[error("metrics: original and reconstructed lengths differ ({a} vs {b})")]
    LengthMismatch { a: usize, b: usize },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

No `.unwrap()` in non-test code. Every decode/encode path returns `Result`.

### 1.3 Codec trait

```rust
// atsc/src/codec/mod.rs
pub trait Codec: Send + Sync {
    fn id(&self) -> u8;
    fn name(&self) -> &'static str;

    fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame>;
    fn decompress(&self, payload: &[u8], sample_count: u32) -> Result<Vec<f64>>;
}

#[derive(Clone, Debug)]
pub struct CompressConfig {
    pub max_error: Option<f64>,
    pub max_iterations: u32,
}

impl Default for CompressConfig {
    fn default() -> Self {
        Self { max_error: Some(0.05), max_iterations: 22 }
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
pub fn decode_stream(buf: &[u8]) -> Result<(Header, Vec<CompressedFrame>)>;
```

All reads are bounds-checked, returning `Error::UnexpectedEof` on truncation. Little-endian throughout.

### 1.5 Feature flags

```toml
[features]
default = []
rayon = ["dep:rayon"]
fft-f64 = []
```

---

## Phase 2 — Codecs (migrate + fix)

### 2.1 FFT

Port the algorithm from `atsc/src/compressor/fft.rs` into `atsc/src/codec/fft.rs`, implementing `Codec`.

Fixes:
- **f64_to_f32**: return `Error::PrecisionOverflow` on non-finite result.
- **Bounded loop**: use `(best_error - target).abs() < epsilon` instead of truncated integer comparison.
- **Dedup stats**: single `DataStats::new(data)` call, remove manual min/max loops.
- **Gibbs sizing**: keep the pad-with-edge-values approach; it works.
- **f64 mode**: when feature `fft-f64` is enabled (or config flag set), store `FrequencyPoint` with f64 real/imag and use codec ID 3.
- **Manual payload encode/decode**: replace bincode with hand-written LE byte serialization.

### 2.2 Polynomial / IDW

Port into `atsc/src/codec/polynomial.rs`, implementing `Codec`.

Fixes:
- **`point_step`**: `u32` (was `u8` — critical overflow bug).
- **Unify types**: single `InterpolationMethod` enum replaces `PolynomialType` + `Method`.
- **Bounded loop fallback**: return `Error::ErrorBoundNotMet` instead of storing the entire dataset.
- **Manual payload encode/decode**: bitdepth-aware, same as before but hand-written.

### 2.3 Constant

Port into `atsc/src/codec/constant.rs`. Simplest codec.

- Payload: 8 bytes (one f64 LE).
- Remove unused constructor parameters.

### 2.4 Noop

Port into `atsc/src/codec/noop.rs`.

- **Make truly lossless**: store raw f64 LE bytes. No rounding. `payload_len = sample_count * 8`.
- This is the "I don't want any loss" escape hatch.

### 2.5 VSRI

Port `lib_vsri` into `atsc/src/vsri/`.

Fixes:
- **`get_time` bug**: `y = y0 + m * (x - x0)`.
- **Named struct**: `Segment { rate: i64, x0: i64, y0: i64, count: u32 }`.
- **i64 everywhere**: no 2038 problem.
- **Drop `index_file`**: not part of data; persistence is the caller's concern.
- **Drop `day_elapsed_seconds` / `start_day_ts` / `MAX_INDEX_SAMPLES`**: not VSRI concerns.
- **Proper errors**: `Error::VsriOutOfOrder` with context.
- **Manual encode/decode**: `Segment` array to/from bytes.

### 2.6 Error metrics

New `atsc/src/metrics.rs`:

```rust
pub fn nrmse(original: &[f64], reconstructed: &[f64]) -> Result<f64>;
```

- Returns `Result` (no panic on length mismatch).
- NRMSE = `sqrt(MSE) / (max - min)`. Well-defined for all data (no division by zero unless constant — which gets caught by the constant codec anyway).
- Remove MAPE, SMAPE, MAE, MSE as public API. Keep NRMSE as the single metric. Internal helpers can exist but don't need to be public.

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
2. **Try all codecs** on the data (or a subsample if data is large and `compression_speed` allows).
3. **Pick smallest payload** that meets the error bound.
4. If none meet the bound, pick the one with the lowest error.

With `rayon` feature: trial compressions run in parallel.

### 3.2 Chunking

- Power-of-2 chunks for FFT (keep existing logic — it works).
- Allow the chunker to split at detected regime changes (large derivative spikes) in a future iteration. For v2.0 keep the current greedy power-of-2 approach and tune later.
- Min chunk: 512. Max chunk: 131072. Same as today.

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

/// Compress values + timestamps together (VSRI frame prepended).
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
    pub fn finish(self) -> Result<Vec<u8>>;
}

pub struct StreamReader<'a> { ... }
impl<'a> StreamReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self>;
    pub fn header(&self) -> &Header;
    pub fn frames(&self) -> FrameIter<'a>;
}

// === Inspect ===

pub fn inspect(bytes: &[u8]) -> Result<StreamInfo>;

pub struct StreamInfo {
    pub version: u8,
    pub frame_count: u16,
    pub frames: Vec<FrameInfo>,
}

pub struct FrameInfo {
    pub codec_name: &'static str,
    pub sample_count: u32,
    pub payload_bytes: u32,
}
```

### 4.2 CLI (`atsc-cli`)

```
atsc compress <input.wbro> [-o output.atsc] [--codec auto|fft|poly|const|noop|idw] [--error 5]
atsc decompress <input.atsc> [-o output.wbro]
atsc inspect <input.atsc>
atsc bench <input.wbro>     # runs all codecs, prints table of ratio/error/time
```

Thin wrapper. Reads `.wbro` files (WavBrro format) for input compatibility with existing test data. Output is `.atsc` (new format). The CLI does file I/O; the library never does.

---

## Phase 5 — Quality

### 5.1 Testing

- **Roundtrip property tests** (`proptest`): for every codec, `decompress(compress(data))` is within error bound for random data.
- **Fuzz targets** (`cargo-fuzz`): feed random bytes to `decompress()`, `decode_stream()`, `decode_header()`. Must never panic.
- **Wire format tests**: hand-craft byte sequences, verify decode. Encode then decode, verify roundtrip.
- **Regression benchmarks** (Criterion): FFT compress/decompress at 256/1024/4096/65536 sizes. Polynomial same.

### 5.2 Dependencies (target)

```toml
# atsc/Cargo.toml
[dependencies]
thiserror = "2"
rustfft = "6.1"
splines = "4.3"
inverse_distance_weight = "0.1"
log = "0.4"
rayon = { version = "1", optional = true }

[dev-dependencies]
criterion = "0.5"
proptest = "1"
```

That's it. No bincode, no rkyv, no hound, no regex, no average, no median, no chrono, no tempfile, no csv.

### 5.3 CI

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -D warnings` (no allows)
- `cargo test --workspace`
- `cargo test --workspace --features rayon`
- `cargo test --workspace --features fft-f64`
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
| 1 | Workspace restructure: create `atsc/` (lib) + `atsc-cli/` (bin), delete dropped crates | — |
| 2 | `error.rs`: define `Error` enum + `Result` alias | 1 |
| 3 | `codec/mod.rs`: define `Codec` trait, `CompressConfig`, `CompressedFrame` | 2 |
| 4 | `format/`: implement manual wire format (header + frame encode/decode) | 2, 3 |
| 5 | `metrics.rs`: implement NRMSE | 2 |
| 6 | `codec/constant.rs`: implement Constant codec | 3, 4 |
| 7 | `codec/noop.rs`: implement Noop codec (truly lossless) | 3, 4 |
| 8 | `codec/fft.rs`: port + fix FFT codec | 3, 4, 5 |
| 9 | `codec/polynomial.rs`: port + fix Polynomial/IDW codec | 3, 4, 5 |
| 10 | `vsri/`: port + fix VSRI with i64 segments | 2, 4 |
| 11 | `optimizer/`: implement chunker + codec selection | 3, 5, 6–9 |
| 12 | Public API: `compress()`, `decompress()`, `StreamWriter`, `StreamReader` | 4, 11 |
| 13 | `atsc-cli`: build CLI with subcommands | 12 |
| 14 | Property tests + fuzz targets + wire format tests | 12 |
| 15 | CI setup + lint cleanup + dependency audit | 13, 14 |
