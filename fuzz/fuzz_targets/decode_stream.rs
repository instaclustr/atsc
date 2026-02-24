#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > (1 << 20) {
        return;
    }
    let _ = atsc::format::stream::decode_stream(data);
});

