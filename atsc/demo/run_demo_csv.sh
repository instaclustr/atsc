#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 INPUT.csv [--strict]"
  echo "expects a single numeric value per line (first column)."
  exit 2
fi

INPUT="$1"
STRICT=0
if [[ "${2:-}" == "--strict" ]]; then
  STRICT=1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUT_DIR="$SCRIPT_DIR/generated"

if [[ ! -f "$INPUT" ]]; then
  echo "error: input file not found: $INPUT"
  exit 2
fi

STEM="$(basename "$INPUT")"
STEM="${STEM%.csv}"
NAME="$STEM"
if [[ "$STEM" == *iowait* ]]; then NAME="csv-iowait"; fi

TMP="$(mktemp -d "$SCRIPT_DIR/.tmp-demo-$NAME-XXXXXX")"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

mkdir -p "$OUT_DIR"

python3 - "$INPUT" "$TMP/input.f64" <<'PY'
import csv, struct, sys
inp, outp = sys.argv[1:3]
vals = []
with open(inp, newline="") as f:
    for row in csv.reader(f):
        if not row:
            continue
        vals.append(float(row[0]))
with open(outp, "wb") as f:
    for v in vals:
        f.write(struct.pack("<d", v))
print(f"wrote {len(vals)} samples to {outp}")
PY

cd "$ROOT"
CARGO_TARGET_DIR="$ROOT/target" cargo build -q --locked -p atsc-cli
ATSC="$ROOT/target/debug/atsc-cli"

STRICT_FLAG=()
if [[ "$STRICT" -eq 1 ]]; then
  STRICT_FLAG=(--strict)
fi

for ERR in 1 3; do
  for CODEC in auto fft poly; do
    "$ATSC" compress "$TMP/input.f64" --codec "$CODEC" --error "$ERR" --iters 200 "${STRICT_FLAG[@]}" -o "$TMP/$CODEC.atsc"
    "$ATSC" decompress "$TMP/$CODEC.atsc" -o "$TMP/$CODEC.f64"
  done

  python3 - "$ERR" "$TMP/input.f64" "$TMP/auto.f64" "$TMP/fft.f64" "$TMP/poly.f64" "$OUT_DIR/comparison-v2-error-$ERR-$NAME.html" <<'PY'
import json, struct, sys

def read_f64(path: str):
    b = open(path, "rb").read()
    if len(b) % 8 != 0:
        raise SystemExit(f"{path}: not a multiple of 8 bytes")
    n = len(b) // 8
    return list(struct.unpack("<%sd" % n, b))

err_level, inp_path, auto_path, fft_path, poly_path, out_path = sys.argv[1:7]
inp  = read_f64(inp_path)
auto = read_f64(auto_path)
fft  = read_f64(fft_path)
poly = read_f64(poly_path)

inp_js = json.dumps(inp, allow_nan=True)
auto_js = json.dumps(auto, allow_nan=True)
fft_js = json.dumps(fft, allow_nan=True)
poly_js = json.dumps(poly, allow_nan=True)

html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Comparison Error Level {err_level}</title>
</head>
<body>
<div id="controls" style="font-family: sans-serif; margin-bottom: 12px;"></div>
<canvas id="myChart" width="1400" height="500" style="border: 1px solid #ddd;"></canvas>
<script>
const inputData = {inp_js};
const autoData = {auto_js};
const fftData = {fft_js};
const polyData = {poly_js};

const series = [
  {{ name: 'Data', color: 'green', data: inputData, enabled: true }},
  {{ name: 'Auto', color: 'purple', data: autoData, enabled: true }},
  {{ name: 'FFT', color: 'red', data: fftData, enabled: true }},
  {{ name: 'Poly', color: 'black', data: polyData, enabled: true }},
];

function minMaxEnabled(maxSamples = 5000) {{
  let min = Number.POSITIVE_INFINITY;
  let max = Number.NEGATIVE_INFINITY;
  for (const s of series) {{
    if (!s.enabled) continue;
    const step = Math.max(1, Math.ceil(s.data.length / maxSamples));
    for (let i = 0; i < s.data.length; i += step) {{
      const v = s.data[i];
      if (!Number.isFinite(v)) continue;
      if (v < min) min = v;
      if (v > max) max = v;
    }}
  }}
  if (!Number.isFinite(min) || !Number.isFinite(max)) return [0, 1];
  if (min === max) return [min - 1, max + 1];
  return [min, max];
}}

function draw() {{
  const canvas = document.getElementById('myChart');
  const ctx = canvas.getContext('2d');
  const w = canvas.width;
  const h = canvas.height;
  const margin = 40;
  const plotW = w - margin * 2;
  const plotH = h - margin * 2;

  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, w, h);

  const [minY, maxY] = minMaxEnabled();
  const n = inputData.length;
  const denomX = Math.max(1, n - 1);
  const denomY = (maxY - minY);

  // axes
  ctx.strokeStyle = '#999';
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(margin, margin);
  ctx.lineTo(margin, h - margin);
  ctx.lineTo(w - margin, h - margin);
  ctx.stroke();

  // labels
  ctx.fillStyle = '#333';
  ctx.font = '12px sans-serif';
  ctx.fillText(maxY.toFixed(3), 4, margin + 4);
  ctx.fillText(minY.toFixed(3), 4, h - margin);

  for (const s of series) {{
    if (!s.enabled) continue;
    const step = Math.max(1, Math.ceil(s.data.length / plotW));
    ctx.strokeStyle = s.color;
    ctx.lineWidth = 1;
    ctx.beginPath();
    let started = false;
    for (let i = 0; i < s.data.length; i += step) {{
      const v = s.data[i];
      if (!Number.isFinite(v)) continue;
      const x = margin + (i / denomX) * plotW;
      const y = margin + (1 - (v - minY) / denomY) * plotH;
      if (!started) {{
        ctx.moveTo(x, y);
        started = true;
      }} else {{
        ctx.lineTo(x, y);
      }}
    }}
    ctx.stroke();
  }}
}}

const controls = document.getElementById('controls');
for (const s of series) {{
  const label = document.createElement('label');
  label.style.marginRight = '12px';
  const cb = document.createElement('input');
  cb.type = 'checkbox';
  cb.checked = s.enabled;
  cb.addEventListener('change', () => {{
    s.enabled = cb.checked;
    draw();
  }});
  const swatch = document.createElement('span');
  swatch.textContent = '■';
  swatch.style.color = s.color;
  swatch.style.margin = '0 6px';
  label.appendChild(cb);
  label.appendChild(swatch);
  label.appendChild(document.createTextNode(s.name));
  controls.appendChild(label);
}}

draw();
</script>
</body>
</html>
"""

open(out_path, "w").write(html)
print(f"wrote {out_path}")
PY
done

echo "done"

