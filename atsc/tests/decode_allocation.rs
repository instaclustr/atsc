// Implementing `GlobalAlloc` requires unsafe code.
#![allow(unsafe_code)]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

use atsc::{data::CompressedStream, error::DecodeError};

/// Records the largest single allocation so the test can prove that a forged
/// length prefix is rejected before bincode zero-fills a buffer for it. This
/// binary must contain only one test because the counter is process-wide.
struct LargestAllocation;

static LARGEST: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for LargestAllocation {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
            System.alloc(layout)
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe {
            LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
            System.alloc_zeroed(layout)
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            LARGEST.fetch_max(new_size, Ordering::Relaxed);
            System.realloc(ptr, layout, new_size)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: LargestAllocation = LargestAllocation;

const FORGED_PAYLOAD_LEN: u32 = (1 << 28) - 64;
const ALLOCATION_BUDGET: usize = 1024 * 1024;

fn stream_with_forged_payload_length() -> Vec<u8> {
    let mut bytes = b"BRRO".to_vec();
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.push(1);
    // frame vector length, frame_size, sample_count, Noop codec
    bytes.extend_from_slice(&[1, 0, 1, 0]);
    // varint u32 marker followed by the payload length; no payload bytes follow
    bytes.push(252);
    bytes.extend_from_slice(&FORGED_PAYLOAD_LEN.to_le_bytes());
    bytes
}

#[test]
fn forged_frame_payload_length_is_rejected_without_a_large_allocation() {
    let bytes = stream_with_forged_payload_length();
    LARGEST.store(0, Ordering::Relaxed);

    let error = CompressedStream::try_from_bytes(&bytes)
        .expect_err("a payload length without payload bytes must be rejected");

    let largest = LARGEST.load(Ordering::Relaxed);
    assert!(
        matches!(
            error,
            DecodeError::Bincode {
                context: "BRO frame vector",
                source: bincode::error::DecodeError::UnexpectedEnd { .. },
            }
        ),
        "{error:?}"
    );
    assert!(
        largest <= ALLOCATION_BUDGET,
        "decoding an {}-byte stream allocated {largest} bytes at once",
        bytes.len()
    );
}
