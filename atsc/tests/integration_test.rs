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
