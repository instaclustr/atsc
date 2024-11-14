use atsc::csv::{read_samples, read_samples_with_headers};
use atsc::utils::error::calculate_error;
use std::fs;
use std::path::{Path, PathBuf};
use wavbrro::wavbrro::WavBrro;

const TEST_FILE_NAME: &str = "go_gc_heap_goal_bytes.wbro";
const TEST_COMPRESSED_FILE_NAME: &str = "go_gc_heap_goal_bytes.bro";
const TEST_WBRO_PATH: &str = "tests/wbros/go_gc_heap_goal_bytes.wbro";

#[test]
fn test_compressor_idw_lossless() {
    test_lossless_compression("idw")
}

#[test]
fn test_compressor_idw_lossy() {
    test_lossy_compression("idw")
}

#[test]
fn test_compressor_polynomial_lossless() {
    test_lossless_compression("polynomial")
}

#[test]
fn test_compressor_polynomial_lossy() {
    test_lossy_compression("polynomial")
}

#[test]
fn test_compressor_noop() {
    test_lossless_compression("noop")
}

#[test]
fn test_compressor_fft_lossy() {
    test_lossy_compression("fft")
}

#[test]
fn test_compressor_auto_lossless() {
    test_lossless_compression("auto")
}

#[test]
fn test_compressor_auto_lossy() {
    test_lossy_compression("auto")
}

#[test]
fn test_csv_input_compression_with_header() {
    let filepath = Path::new("./tests/csv/cpu_utilization.csv");
    let test_dir = prepare_test_dir_and_copy_file(filepath);
    let csv_file_path = test_dir.join("cpu_utilization.csv");

    run_compressor(&[
        "--compressor",
        "noop",
        "--error",
        "5",
        "--csv",
        "--fields=time,value",
        csv_file_path.to_str().unwrap(),
    ]);

    run_compressor(&["-u", test_dir.join("cpu_utilization.bro").to_str().unwrap()]);

    let uncompressed_samples = WavBrro::from_file(&test_dir.join("cpu_utilization.wbro")).unwrap();
    let original_samples = read_values_from_csv(&csv_file_path, false);

    let err = calculate_error(&original_samples, &uncompressed_samples);

    assert!(
        err <= 0.05,
        "Error: {}\nOriginal    : {:?}\nUncompressed: {:?}",
        err,
        original_samples,
        uncompressed_samples
    );
}

#[test]
fn test_csv_input_compression_with_header_no_fields() {
    let filepath = Path::new("./tests/csv/cpu_utilization.csv");
    let test_dir = prepare_test_dir_and_copy_file(filepath);
    let csv_file_path = test_dir.join("cpu_utilization.csv");

    run_compressor(&[
        "--compressor",
        "noop",
        "--error",
        "5",
        "--csv",
        csv_file_path.to_str().unwrap(),
    ]);

    run_compressor(&["-u", test_dir.join("cpu_utilization.bro").to_str().unwrap()]);

    let uncompressed_samples = WavBrro::from_file(&test_dir.join("cpu_utilization.wbro")).unwrap();
    let original_samples = read_values_from_csv(&csv_file_path, false);

    let err = calculate_error(&original_samples, &uncompressed_samples);

    assert!(
        err <= 0.05,
        "Error: {}\nOriginal    : {:?}\nUncompressed: {:?}",
        err,
        original_samples,
        uncompressed_samples
    );
}

#[test]
fn test_csv_input_compression_without_header() {
    let filepath = Path::new("./tests/csv/cpu_utilization_no_headers_only_values.csv");
    let test_dir = prepare_test_dir_and_copy_file(filepath);
    let csv_file_path = test_dir.join("cpu_utilization_no_headers_only_values.csv");

    run_compressor(&[
        "--compressor",
        "noop",
        "--error",
        "5",
        "--csv",
        "--no-header",
        csv_file_path.to_str().unwrap(),
    ]);

    run_compressor(&[
        "-u",
        test_dir
            .join("cpu_utilization_no_headers_only_values.bro")
            .to_str()
            .unwrap(),
    ]);

    let uncompressed_samples =
        WavBrro::from_file(&test_dir.join("cpu_utilization_no_headers_only_values.wbro")).unwrap();
    let original_samples = read_values_from_csv(&csv_file_path, true);

    let err = calculate_error(&original_samples, &uncompressed_samples);

    assert!(
        err <= 0.05,
        "Error: {}\nOriginal    : {:?}\nUncompressed: {:?}",
        err,
        original_samples,
        uncompressed_samples
    );
}

fn test_lossless_compression(compressor: &str) {
    test_compression_decompression_flow(compressor, 0, compare_samples_lossless)
}

fn test_lossy_compression(compressor: &str) {
    test_compression_decompression_flow(compressor, 5, compare_samples_with_allowed_error)
}

#[test]
fn test_compressor_constant() {
    // tests/wbros/uptime.wbro constant data which can be compressed by constant compressor
    let test_dir = tempfile::tempdir().unwrap().into_path();
    fs::copy("tests/wbros/uptime.wbro", test_dir.join("uptime.wbro")).unwrap();

    run_compressor(&[
        "--compressor",
        "noop",
        test_dir.join("uptime.wbro").to_str().unwrap(),
    ]);

    run_compressor(&["-u", test_dir.join("uptime.bro").to_str().unwrap()]);

    compare_samples_lossless(
        &PathBuf::from("tests/wbros/uptime.wbro"),
        &test_dir.join("uptime.wbro"),
    )
}

/// Runs compression and decompression test for a specified compressor.
/// max_error is an error level, compression speed is set as the lowest (0).
///
/// It compresses tests/wbros/go_gc_duration_count.wbro file, decompresses
/// got .bro file, and compares the original .wbro and the decompressed one.
fn test_compression_decompression_flow(
    compressor: &str,
    allowed_error: u8,
    compare_fn: fn(original: &Path, processed: &Path),
) {
    let test_dir = prepare_test_dir();

    // Running data compression
    run_compressor(&[
        "--compressor",
        compressor,
        "--error",
        allowed_error.to_string().as_str(),
        test_dir.join(TEST_FILE_NAME).to_str().unwrap(),
    ]);

    // Running data decompression
    run_compressor(&[
        "-u",
        test_dir.join(TEST_COMPRESSED_FILE_NAME).to_str().unwrap(),
    ]);

    compare_fn(
        &PathBuf::from(TEST_WBRO_PATH),
        &test_dir.join(TEST_FILE_NAME),
    )
}

/// Prepares test directory and copies test wbro file there.
fn prepare_test_dir() -> PathBuf {
    let test_dir = tempfile::tempdir().unwrap().into_path();
    fs::copy(TEST_WBRO_PATH, test_dir.join(TEST_FILE_NAME)).unwrap();
    test_dir
}

/// Prepares test directory and copies file to it.
fn prepare_test_dir_and_copy_file(filepath: &Path) -> PathBuf {
    let test_dir = tempfile::tempdir().unwrap().into_path();
    fs::copy(filepath, test_dir.join(filepath.file_name().unwrap())).unwrap();
    test_dir
}

/// Runs compressor binary with provided arguments.
fn run_compressor(args: &[&str]) {
    let compressor_bin = env!("CARGO_BIN_EXE_atsc");
    let exit_status = std::process::Command::new(compressor_bin)
        .args(args)
        .status()
        .unwrap();

    assert!(exit_status.success());
}

fn compare_samples_lossless(original: &Path, uncompressed: &Path) {
    let original_samples = WavBrro::from_file(original).unwrap();
    let uncompressed_samples = WavBrro::from_file(uncompressed).unwrap();
    assert_eq!(original_samples, uncompressed_samples);
}

fn compare_samples_with_allowed_error(original: &Path, uncompressed: &Path) {
    const MAX_ALLOWED_ERROR: f64 = 0.05;

    let original_samples = WavBrro::from_file(original).unwrap();
    let uncompressed_samples = WavBrro::from_file(uncompressed).unwrap();
    let err = calculate_error(&original_samples, &uncompressed_samples);

    assert!(
        err <= MAX_ALLOWED_ERROR,
        "Error: {}\nOriginal    : {:?}\nUncompressed: {:?}",
        err,
        original_samples,
        uncompressed_samples
    );
}

/// Reads csv file and returns the content of value field.
/// If no_headers = false it assumes --fields="time,value"
fn read_values_from_csv(csv_file_path: &Path, no_headers: bool) -> Vec<f64> {
    let samples = if no_headers {
        read_samples(csv_file_path).unwrap()
    } else {
        read_samples_with_headers(csv_file_path, "time", "value").unwrap()
    };

    samples.into_iter().map(|sample| sample.value).collect()
}
