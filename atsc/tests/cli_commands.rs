use std::{
    ffi::OsStr,
    fs,
    process::{Command, Output},
};

use atsc::compressor::{BinConfig, Compressor};
use serde_json::Value;
use tempfile::tempdir;
use wavbrro::wavbrro::WavBrro;

const EXIT_IO: i32 = 1;
const EXIT_USAGE: i32 = 2;
const EXIT_DECODE: i32 = 3;
const EXIT_ENCODE: i32 = 4;
const EXIT_INPUT_FORMAT: i32 = 5;

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_atsc"))
}

fn run<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    command().args(args).output().expect("CLI must run")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed with {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_constant_fixture(path: &std::path::Path) {
    fs::write(path, include_bytes!("fixtures/v1/constant.bro")).expect("fixture must be written");
}

fn write_wbro(path: &std::path::Path, samples: &[f64]) {
    WavBrro::to_file_with_data(path, samples);
}

#[derive(bincode::Encode)]
struct EncodedFrame {
    frame_size: usize,
    sample_count: usize,
    compressor: Compressor,
    data: Vec<u8>,
}

fn bro_with_invalid_inner_payload() -> Vec<u8> {
    let mut bytes = b"BRRO".to_vec();
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.push(1);
    bincode::encode_into_std_write(
        vec![EncodedFrame {
            frame_size: 0,
            sample_count: 1,
            compressor: Compressor::Auto,
            data: Vec::new(),
        }],
        &mut bytes,
        BinConfig::get(),
    )
    .expect("invalid inner fixture must encode");
    bytes
}

#[test]
fn inspect_prints_stream_and_frame_metadata() {
    let temp = tempdir().unwrap();
    let fixture = temp.path().join("fixture.bro");
    write_constant_fixture(&fixture);

    let output = command().arg("inspect").arg(&fixture).output().unwrap();

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("version: 1"), "{stdout}");
    assert!(stdout.contains("frames: 1"), "{stdout}");
    assert!(stdout.contains("samples: 16"), "{stdout}");
    assert!(stdout.contains("codec=constant"), "{stdout}");
    assert!(stdout.contains("payload bytes="), "{stdout}");
}

#[test]
fn inspect_json_is_one_machine_readable_document() {
    let temp = tempdir().unwrap();
    let fixture = temp.path().join("fixture.bro");
    write_constant_fixture(&fixture);

    let output = command()
        .arg("inspect")
        .arg(&fixture)
        .arg("--json")
        .output()
        .unwrap();

    assert_success(&output);
    assert!(output.stderr.is_empty());
    let document: Value =
        serde_json::from_slice(&output.stdout).expect("stdout must contain only one JSON document");
    assert_eq!(document["version"], 1);
    assert_eq!(document["frames"], 1);
    assert_eq!(document["samples"], 16);
    assert_eq!(document["frame_details"][0]["codec"], "constant");
    assert!(
        document["frame_details"][0]["payload_bytes"]
            .as_u64()
            .unwrap()
            > 0
    );
}

#[test]
fn verify_valid_stream_prints_valid() {
    let temp = tempdir().unwrap();
    let fixture = temp.path().join("fixture.bro");
    write_constant_fixture(&fixture);

    let output = command().arg("verify").arg(&fixture).output().unwrap();

    assert_success(&output);
    assert_eq!(output.stdout, b"valid\n");
}

#[test]
fn verify_truncated_stream_returns_typed_decode_error() {
    let temp = tempdir().unwrap();
    let fixture = temp.path().join("truncated.bro");
    let mut bytes = include_bytes!("fixtures/v1/constant.bro").to_vec();
    bytes.pop();
    fs::write(&fixture, bytes).unwrap();

    let output = command().arg("verify").arg(&fixture).output().unwrap();

    assert_eq!(output.status.code(), Some(EXIT_DECODE));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("failed to decode BRO frame vector"),
        "{stderr}"
    );
}

#[test]
fn verify_fully_decodes_inner_payloads() {
    let temp = tempdir().unwrap();
    let fixture = temp.path().join("inner-corruption.bro");
    fs::write(&fixture, bro_with_invalid_inner_payload()).unwrap();

    let inspection = command().arg("inspect").arg(&fixture).output().unwrap();
    assert_success(&inspection);

    let verification = command().arg("verify").arg(&fixture).output().unwrap();
    assert_eq!(verification.status.code(), Some(EXIT_DECODE));
    let stderr = String::from_utf8_lossy(&verification.stderr);
    assert!(
        stderr.contains("Auto is not a stored frame codec"),
        "{stderr}"
    );
}

#[test]
fn explicit_compress_and_decompress_round_trip_to_exact_output_paths() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.wbro");
    let compressed = temp.path().join("chosen-name.bro");
    let restored = temp.path().join("restored.wbro");
    let samples = [1.0, 2.0, 2.0, 4.0];
    write_wbro(&input, &samples);

    let compression = command()
        .arg("compress")
        .arg(&input)
        .arg("-o")
        .arg(&compressed)
        .arg("--compressor")
        .arg("noop")
        .output()
        .unwrap();
    assert_success(&compression);
    assert!(compressed.is_file());
    assert!(!input.with_extension("bro").exists());

    let decompression = command()
        .arg("decompress")
        .arg(&compressed)
        .arg("-o")
        .arg(&restored)
        .output()
        .unwrap();
    assert_success(&decompression);
    assert_eq!(WavBrro::from_file(&restored).unwrap(), samples);
}

#[test]
fn legacy_wbro_and_uncompress_invocations_still_round_trip() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("legacy.wbro");
    let samples = [2.0, 2.0, 5.0, 5.0];
    write_wbro(&input, &samples);

    let compression = command()
        .arg(&input)
        .arg("--compressor")
        .arg("rle")
        .output()
        .unwrap();
    assert_success(&compression);

    let compressed = input.with_extension("bro");
    fs::remove_file(&input).unwrap();
    let decompression = command().arg("-u").arg(&compressed).output().unwrap();
    assert_success(&decompression);
    assert_eq!(WavBrro::from_file(&input).unwrap(), samples);
}

#[test]
fn legacy_csv_headers_fields_and_rle_are_preserved() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("series.csv");
    fs::write(&input, "when,reading\n1,4\n2,4\n3,7\n").unwrap();

    let compression = command()
        .args(["--csv", "--fields=when,reading", "--compressor", "rle"])
        .arg(&input)
        .output()
        .unwrap();
    assert_success(&compression);

    let compressed = input.with_extension("bro");
    let decompression = command().arg("-u").arg(&compressed).output().unwrap();
    assert_success(&decompression);
    assert_eq!(
        WavBrro::from_file(&input.with_extension("wbro")).unwrap(),
        [4.0, 4.0, 7.0]
    );
}

#[test]
fn legacy_help_keeps_default_error_and_rle() {
    let output = run(["--help"]);

    assert_success(&output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("[default: 3]"), "{stdout}");
    assert!(stdout.contains("rle"), "{stdout}");
}

#[test]
fn directory_compression_filters_initial_entries_and_keeps_names() {
    let temp = tempdir().unwrap();
    let directory = temp.path().join("inputs");
    fs::create_dir(&directory).unwrap();
    write_wbro(&directory.join("series.wbro"), &[1.0, 2.0, 3.0]);
    let existing_bro = directory.join("existing.bro");
    write_constant_fixture(&existing_bro);
    let existing_bytes = fs::read(&existing_bro).unwrap();
    fs::write(directory.join("ignored.txt"), "not an input").unwrap();

    let output = command()
        .arg("compress")
        .arg(&directory)
        .arg("--compressor")
        .arg("noop")
        .output()
        .unwrap();

    assert_success(&output);
    assert!(directory.join("series.bro").is_file());
    assert_eq!(fs::read(existing_bro).unwrap(), existing_bytes);
    assert!(!directory.join("existing.wbro").exists());
    assert!(!directory.join("ignored.bro").exists());
}

#[test]
fn directory_failures_are_aggregated_without_retrying_entries() {
    let temp = tempdir().unwrap();
    let directory = temp.path().join("inputs");
    fs::create_dir(&directory).unwrap();
    write_wbro(&directory.join("good.wbro"), &[1.0, 2.0, 3.0]);
    fs::write(directory.join("bad.wbro"), b"XXXXXXXXXXXX").unwrap();

    let output = command()
        .arg("compress")
        .arg(&directory)
        .arg("--compressor")
        .arg("noop")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(EXIT_INPUT_FORMAT));
    assert!(directory.join("good.bro").is_file());
    assert!(!directory.join("bad.bro").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.matches("bad.wbro").count(), 1, "{stderr}");
    assert!(stderr.contains("1 file failed"), "{stderr}");
}

#[test]
fn output_option_is_rejected_for_directory_input() {
    let temp = tempdir().unwrap();
    let directory = temp.path().join("inputs");
    fs::create_dir(&directory).unwrap();
    write_wbro(&directory.join("series.wbro"), &[1.0]);
    let output_path = temp.path().join("single.bro");

    let output = command()
        .arg("compress")
        .arg(&directory)
        .arg("-o")
        .arg(&output_path)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(EXIT_USAGE));
    assert!(!output_path.exists());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("--output can only be used with a single input file"));
}

#[test]
fn io_errors_use_the_stable_io_exit_code() {
    let temp = tempdir().unwrap();
    let missing = temp.path().join("missing.bro");

    let output = command().arg("verify").arg(&missing).output().unwrap();

    assert_eq!(output.status.code(), Some(EXIT_IO));
    assert!(!output.stderr.is_empty());
}

#[test]
fn encode_errors_are_reported_without_panicking_in_new_and_legacy_modes() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("nonconstant.wbro");
    write_wbro(&input, &[1.0, 2.0]);
    let explicit_output = temp.path().join("explicit.bro");

    let explicit = command()
        .arg("compress")
        .arg(&input)
        .arg("-o")
        .arg(&explicit_output)
        .arg("--compressor")
        .arg("constant")
        .output()
        .unwrap();
    assert_eq!(explicit.status.code(), Some(EXIT_ENCODE));
    assert!(!explicit_output.exists());
    let explicit_stderr = String::from_utf8_lossy(&explicit.stderr);
    assert!(
        explicit_stderr.contains("Constant compression did not meet error bound"),
        "{explicit_stderr}"
    );
    assert!(
        !explicit_stderr.contains("panicked at"),
        "{explicit_stderr}"
    );

    let legacy = command()
        .arg(&input)
        .arg("--compressor")
        .arg("constant")
        .output()
        .unwrap();
    assert_eq!(legacy.status.code(), Some(EXIT_ENCODE));
    assert!(!input.with_extension("bro").exists());
    let legacy_stderr = String::from_utf8_lossy(&legacy.stderr);
    assert!(
        legacy_stderr.contains("Constant compression did not meet error bound"),
        "{legacy_stderr}"
    );
    assert!(!legacy_stderr.contains("panicked at"), "{legacy_stderr}");
}

#[test]
fn invalid_wbro_uses_the_stable_input_format_exit_code() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("invalid.wbro");
    fs::write(&input, b"XXXXXXXXXXXX").unwrap();
    let output_path = temp.path().join("output.bro");

    let output = command()
        .arg("compress")
        .arg(&input)
        .arg("-o")
        .arg(&output_path)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(EXIT_INPUT_FORMAT));
    assert!(!output_path.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Wrong WAVBRRO file"));
}
