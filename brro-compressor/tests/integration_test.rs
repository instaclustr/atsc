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

fn test_suite(compressor: &str) {
    compress_dir(compressor);
    compress_file(compressor);
}

fn compress_dir(compressor: &str) {
    let tmp_dir = tempdir().unwrap();
    let input = tmp_dir.path().join("input");
    let output = tmp_dir.path().join("input-compressed");
    std::fs::create_dir(&input).unwrap();
    std::fs::copy("tests/wavs/memory_used.wav", input.join("1.wav")).unwrap();
    std::fs::copy("tests/wavs/uptime.wav", input.join("2.wav")).unwrap();

    run_compressor(&[input.to_str().unwrap(), "--compressor", compressor]);
    assert!(output.join("1.bro").is_file());
    assert!(output.join("2.bro").is_file());
}

fn compress_file(compressor: &str) {
    let tmp_dir = tempdir().unwrap();
    let path = tmp_dir.path();
    std::fs::copy("tests/wavs/memory_used.wav", path.join("1.wav")).unwrap();

    run_compressor(&[
        path.join("1.wav").to_str().unwrap(),
        "--compressor",
        compressor,
    ]);
    assert!(path.join("1.bro").is_file());
}

fn run_compressor(args: &[&str]) {
    // path to binary set by cargo: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    let command = std::env!("CARGO_BIN_EXE_brro-compressor");

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
