use atsc::{compressor::Compressor, data::CompressedStream, optimizer::OptimizerPlan};

const SINE_SAMPLE_COUNT: usize = 131_072;
const STEPPED_SAMPLE_COUNT: usize = 131_072;
const SLOWLY_CHANGING_SAMPLE_COUNT: usize = 262_144;
const MAX_ERROR: f32 = 0.03;
const AUTO_SELECTION_SAMPLE_LEVEL: usize = 1;
const TASK_4_FORCED_FFT_SINE_BYTES: usize = 13_996;
const TASK_4_STEPPED_AUTO_BYTES: usize = 2_101;
const TASK_4_SLOWLY_CHANGING_AUTO_BYTES: usize = 1_568;

fn sine_samples() -> Vec<f64> {
    (0..SINE_SAMPLE_COUNT)
        .map(|index| ((index as f64) / 8.0).sin())
        .collect()
}

fn stepped_samples() -> Vec<f64> {
    (0..STEPPED_SAMPLE_COUNT)
        .map(|index| ((index / 256) % 16) as f64)
        .collect()
}

fn slowly_changing_samples() -> Vec<f64> {
    (0..SLOWLY_CHANGING_SAMPLE_COUNT)
        .map(|index| (index / 1_024) as f64)
        .collect()
}

fn encode_optimizer_stream(samples: &[f64], compressor: Compressor) -> Vec<u8> {
    let mut plan = OptimizerPlan::plan(samples);
    plan.set_compressor(compressor);

    let mut stream = CompressedStream::new();
    for (selected, chunk) in plan.get_execution() {
        stream.compress_chunk_bounded_with(
            chunk,
            *selected,
            MAX_ERROR,
            AUTO_SELECTION_SAMPLE_LEVEL,
        );
    }
    assert_eq!(stream.sample_count(), samples.len());
    stream.to_bytes()
}

fn assert_within_one_percent(case: &str, baseline_bytes: usize, actual_bytes: usize) {
    let maximum_bytes = (baseline_bytes * 101).div_ceil(100);
    let percentage_change = ((actual_bytes as f64 / baseline_bytes as f64) - 1.0) * 100.0;

    println!(
        "{case}: old={baseline_bytes} bytes, new={actual_bytes} bytes, change={percentage_change:+.3}%"
    );
    assert!(
        actual_bytes <= maximum_bytes,
        "{case}: old={baseline_bytes} bytes, new={actual_bytes} bytes, \
         change={percentage_change:+.3}%, 1% limit={maximum_bytes} bytes"
    );
}

#[test]
fn forced_fft_sine_size_stays_within_one_percent() {
    let actual_bytes = encode_optimizer_stream(&sine_samples(), Compressor::FFT).len();
    assert_within_one_percent(
        "131072-point forced FFT sine",
        TASK_4_FORCED_FFT_SINE_BYTES,
        actual_bytes,
    );
}

#[test]
fn stepped_auto_size_stays_within_one_percent() {
    let actual_bytes = encode_optimizer_stream(&stepped_samples(), Compressor::Auto).len();
    assert_within_one_percent(
        "131072-point stepped Auto",
        TASK_4_STEPPED_AUTO_BYTES,
        actual_bytes,
    );
}

#[test]
fn slowly_changing_auto_size_stays_within_one_percent() {
    let actual_bytes = encode_optimizer_stream(&slowly_changing_samples(), Compressor::Auto).len();
    assert_within_one_percent(
        "262144-point slowly changing Auto",
        TASK_4_SLOWLY_CHANGING_AUTO_BYTES,
        actual_bytes,
    );
}
