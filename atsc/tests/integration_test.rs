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

use tempfile::tempdir;
use wavbrro::wavbrro::WavBrro;

#[test]
fn test_noop() {
    test_suite("noop");
}

#[test]
fn test_constant() {
    compress_constant_file_and_directory();
}

#[test]
fn test_fft() {
    reject_file_above_bound("fft");
}

#[test]
fn test_idw() {
    test_bounded_suite("idw");
}

#[test]
fn test_polynomial() {
    test_bounded_suite("polynomial");
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
    compress_file(compressor);
}

fn test_bounded_suite(compressor: &str) {
    compress_bounded_dir(compressor);
    compress_file(compressor);
}

fn reject_file_above_bound(compressor: &str) {
    let tmp_dir = tempdir().unwrap();
    let path = tmp_dir.path();
    let input = path.join("1.wbro");
    std::fs::copy("tests/wbros/memory_used.wbro", &input).unwrap();

    let command = std::env!("CARGO_BIN_EXE_atsc");
    let output = std::process::Command::new(command)
        .args([input.to_str().unwrap(), "--compressor", compressor])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(4));
    assert!(
        stderr.contains("FFT compression did not meet error bound"),
        "{stderr}"
    );
    assert!(!stderr.contains("panicked at"), "{stderr}");
    assert!(!path.join("1.bro").exists());
}

fn compress_constant_file_and_directory() {
    let tmp_dir = tempdir().unwrap();
    let input = tmp_dir.path().join("input");
    std::fs::create_dir(&input).unwrap();
    WavBrro::to_file_with_data(&input.join("1.wbro"), &[7.0; 32]);
    WavBrro::to_file_with_data(&input.join("2.wbro"), &[9.0; 32]);

    run_compressor(&[input.to_str().unwrap(), "--compressor", "constant"]);
    assert!(input.join("1.bro").is_file());
    assert!(input.join("2.bro").is_file());

    let single = tmp_dir.path().join("single.wbro");
    WavBrro::to_file_with_data(&single, &[11.0; 32]);
    run_compressor(&[single.to_str().unwrap(), "--compressor", "constant"]);
    assert!(single.with_extension("bro").is_file());
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
    assert!(input.join("1.bro").is_file());
    assert!(input.join("2.bro").is_file());
}

fn compress_bounded_dir(compressor: &str) {
    let tmp_dir = tempdir().unwrap();
    let input = tmp_dir.path().join("input");
    std::fs::create_dir(&input).unwrap();
    // uptime.wbro contains zeros, so its source-relative MAPE is undefined.
    // Use two bounded-valid fixtures to retain multi-file directory coverage.
    std::fs::copy("tests/wbros/memory_used.wbro", input.join("1.wbro")).unwrap();
    std::fs::copy("tests/wbros/memory_used.wbro", input.join("2.wbro")).unwrap();

    run_compressor(&[input.to_str().unwrap(), "--compressor", compressor]);
    assert!(input.join("1.bro").is_file());
    assert!(input.join("2.bro").is_file());
}

fn compress_file(compressor: &str) {
    let tmp_dir = tempdir().unwrap();
    let path = tmp_dir.path();
    std::fs::copy("tests/wbros/memory_used.wbro", path.join("1.wbro")).unwrap();

    run_compressor(&[
        path.join("1.wbro").to_str().unwrap(),
        "--compressor",
        compressor,
    ]);
    assert!(path.join("1.bro").is_file());
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
    assert!(path.join("1.bro").is_file());
}

fn run_compressor(args: &[&str]) {
    // path to binary set by cargo: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    let command = std::env!("CARGO_BIN_EXE_atsc");

    let status = std::process::Command::new(command)
        .args(args)
        .status()
        .unwrap();

    if !status.success() {
        panic!(
            "Failed to run command {} {:?}, exited with {:?}",
            command, args, status
        );
    }
}
