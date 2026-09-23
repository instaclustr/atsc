/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::path::Path;

use tempfile::tempdir;
use wavbrro::wavbrro::WavBrro;

const REAL_FIXTURES: [&str; 3] = [
    "tests/wbros/memory_used.wbro",
    "tests/wbros/uptime.wbro",
    "tests/wbros/go_gc_heap_goal_bytes.wbro",
];

#[test]
fn test_noop() {
    test_suite("noop");
}

#[test]
fn test_constant() {
    test_suite("constant");
}

#[test]
fn test_fft() {
    test_suite("fft");
}

#[test]
fn test_idw() {
    test_suite("idw");
}

#[test]
fn test_polynomial() {
    test_suite("polynomial");
}

#[test]
fn test_rle() {
    test_suite("rle");
}

#[test]
fn test_auto() {
    test_suite("auto");
}

#[test]
fn test_compression_speed() {
    test_speed();
}

fn test_suite(compressor: &str) {
    compress_dir(compressor);
    compress_files(compressor);
}

fn test_speed() {
    for speed in 0..7 {
        compress_file_with_speed(speed);
    }
}

fn compress_dir(compressor: &str) {
    let tmp_dir = tempdir().unwrap();
    let input = tmp_dir.path().join("input");
    std::fs::create_dir(&input).unwrap();
    std::fs::copy("tests/wbros/memory_used.wbro", input.join("1.wbro")).unwrap();
    std::fs::copy("tests/wbros/uptime.wbro", input.join("2.wbro")).unwrap();

    run_compressor(&[input.to_str().unwrap(), "--compressor", compressor]);
    assert_valid_legacy_output(&input.join("1.bro"), REAL_FIXTURES[0]);
    assert_valid_legacy_output(&input.join("2.bro"), REAL_FIXTURES[1]);
}

fn compress_files(compressor: &str) {
    for fixture in REAL_FIXTURES {
        let tmp_dir = tempdir().unwrap();
        let path = tmp_dir.path();
        std::fs::copy(fixture, path.join("1.wbro")).unwrap();

        run_compressor(&[
            path.join("1.wbro").to_str().unwrap(),
            "--compressor",
            compressor,
        ]);
        assert_valid_legacy_output(&path.join("1.bro"), fixture);
    }
}

fn compress_file_with_speed(speed: u8) {
    let tmp_dir = tempdir().unwrap();
    let path = tmp_dir.path();
    std::fs::copy("tests/wbros/memory_used.wbro", path.join("1.wbro")).unwrap();

    run_compressor(&[
        path.join("1.wbro").to_str().unwrap(),
        "--compression-selection-sample-level",
        &speed.to_string(),
    ]);
    assert_valid_legacy_output(&path.join("1.bro"), REAL_FIXTURES[0]);
}

/// Legacy mode drops non-finite samples, so the restored series must contain
/// exactly the finite input samples, all of them finite.
fn assert_valid_legacy_output(compressed: &Path, original: &str) {
    let tmp_dir = tempdir().unwrap();
    let round_trip = tmp_dir.path().join("round-trip.bro");
    std::fs::copy(compressed, &round_trip).unwrap();

    run_compressor(&["-u", round_trip.to_str().unwrap()]);

    let restored = WavBrro::from_file(&round_trip.with_extension("wbro")).unwrap();
    let finite_samples = WavBrro::from_file(Path::new(original))
        .unwrap()
        .into_iter()
        .filter(|sample| sample.is_finite())
        .count();
    assert_eq!(restored.len(), finite_samples, "{original}");
    assert!(
        restored.iter().all(|sample| sample.is_finite()),
        "{original}"
    );
}

fn run_compressor(args: &[&str]) {
    // path to binary set by cargo: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    let command = std::env!("CARGO_BIN_EXE_atsc");

    let output = std::process::Command::new(command)
        .args(args)
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "Failed to run command {} {:?}, exited with {:?}: {}",
            command,
            args,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
