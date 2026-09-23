use std::{fs, process::Command};

use tempfile::tempdir;

/// (fixture, codec, -c level, byte length, FNV-1a 64) of the legacy CLI output
/// of main fd6b3c7 (`atsc <fixture>.wbro --compressor <codec> -c <level>`).
const MAIN_OUTPUTS: [(&str, &str, &str, usize, u64); 39] = [
    ("memory_used", "auto", "0", 5307, 0x9d40926f6905087c),
    ("memory_used", "noop", "0", 11534, 0x31ff092a21f76d3c),
    ("memory_used", "fft", "0", 2114, 0x2ff7b721dc752449),
    ("memory_used", "constant", "0", 36, 0x1af79d43283bfa53),
    ("memory_used", "polynomial", "0", 5513, 0x79de8f96289d8d1c),
    ("memory_used", "idw", "0", 5513, 0xb91afc850298b6ca),
    ("memory_used", "rle", "0", 9898, 0x490a33206340f3d5),
    ("uptime", "auto", "0", 36, 0xfb0c95afe5c82742),
    ("uptime", "noop", "0", 2335, 0xb8a1467019baa0db),
    ("uptime", "fft", "0", 267, 0x50254805d318bddd),
    ("uptime", "constant", "0", 28, 0xf1e0938c2783eb3f),
    ("uptime", "polynomial", "0", 66, 0x8868b489ec08f7e3),
    ("uptime", "idw", "0", 66, 0x045e284c0ecab093),
    ("uptime", "rle", "0", 39, 0xa1739b592bd5456a),
    (
        "go_gc_heap_goal_bytes",
        "auto",
        "0",
        2595,
        0xe73af7b09fe20beb,
    ),
    (
        "go_gc_heap_goal_bytes",
        "noop",
        "0",
        14811,
        0x82f94c11a392c436,
    ),
    (
        "go_gc_heap_goal_bytes",
        "fft",
        "0",
        2014,
        0x18630e604e3c37c2,
    ),
    (
        "go_gc_heap_goal_bytes",
        "constant",
        "0",
        49,
        0xe7dd8e24c169cf0f,
    ),
    (
        "go_gc_heap_goal_bytes",
        "polynomial",
        "0",
        2917,
        0x58aa1dc27209c3a3,
    ),
    (
        "go_gc_heap_goal_bytes",
        "idw",
        "0",
        1495,
        0xe9e048732e188e42,
    ),
    (
        "go_gc_heap_goal_bytes",
        "rle",
        "0",
        3353,
        0x6c7be0b48e65b274,
    ),
    ("memory_used", "auto", "1", 5307, 0x9d40926f6905087c),
    ("memory_used", "auto", "2", 5307, 0x9d40926f6905087c),
    ("memory_used", "auto", "3", 5307, 0x9d40926f6905087c),
    ("memory_used", "auto", "4", 5307, 0x9d40926f6905087c),
    ("memory_used", "auto", "5", 5307, 0x9d40926f6905087c),
    ("memory_used", "auto", "6", 2114, 0x2ff7b721dc752449),
    ("uptime", "auto", "1", 36, 0xfb0c95afe5c82742),
    ("uptime", "auto", "2", 36, 0xfb0c95afe5c82742),
    ("uptime", "auto", "3", 36, 0xfb0c95afe5c82742),
    ("uptime", "auto", "4", 36, 0xfb0c95afe5c82742),
    ("uptime", "auto", "5", 36, 0xfb0c95afe5c82742),
    ("uptime", "auto", "6", 36, 0xfb0c95afe5c82742),
    (
        "go_gc_heap_goal_bytes",
        "auto",
        "1",
        2595,
        0xe73af7b09fe20beb,
    ),
    (
        "go_gc_heap_goal_bytes",
        "auto",
        "2",
        2595,
        0xe73af7b09fe20beb,
    ),
    (
        "go_gc_heap_goal_bytes",
        "auto",
        "3",
        2595,
        0xe73af7b09fe20beb,
    ),
    (
        "go_gc_heap_goal_bytes",
        "auto",
        "4",
        2014,
        0x18630e604e3c37c2,
    ),
    (
        "go_gc_heap_goal_bytes",
        "auto",
        "5",
        2781,
        0x9d3abf40602ea27e,
    ),
    (
        "go_gc_heap_goal_bytes",
        "auto",
        "6",
        2781,
        0x9d3abf40602ea27e,
    ),
];

fn fnv1a_64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[test]
fn legacy_cli_output_matches_main_bytes_on_real_fixtures() {
    let mut mismatches = Vec::new();
    for (fixture, codec, level, expected_len, expected_hash) in MAIN_OUTPUTS {
        let temp = tempdir().unwrap();
        let input = temp.path().join("input.wbro");
        fs::copy(format!("tests/wbros/{fixture}.wbro"), &input).unwrap();

        let output = Command::new(env!("CARGO_BIN_EXE_atsc"))
            .arg(&input)
            .args(["--compressor", codec, "-c", level])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{fixture} {codec} -c {level}: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let bytes = fs::read(input.with_extension("bro")).unwrap();
        let actual = (bytes.len(), fnv1a_64(&bytes));
        if actual != (expected_len, expected_hash) {
            mismatches.push(format!(
                "{fixture} {codec} -c {level}: main {expected_len} bytes {expected_hash:#018x}, \
                 got {} bytes {:#018x}",
                actual.0, actual.1
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
