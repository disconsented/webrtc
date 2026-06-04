// Benchmarks the two buffer-packaging strategies introduced in commit b0897a1b for the WebRTC
// data channel read loop.
//
// Feature flags (mutually exclusive):
//
//   `buf-copy` (default): pre-commit — allocates `n` bytes and memcpy's the valid portion.
//                         Correct; the read buffer is reused each iteration.
//   `buf-handoff`:        post-commit — zero-copy mem::replace; hands the old buffer to Bytes
//                         and allocates a fresh 65535-byte buffer for the next read.
//                         See WARN comments in data_channel/mod.rs for the two known bugs.
//
// The benchmark is single-threaded and does NOT involve an async executor or SCTP — it isolates
// the packaging step alone, which is where the CPU/memory tradeoff lives.
//
// Run commands:
//   cargo bench --bench data_channel_buffer -p webrtc --features buf-copy             # pre-commit (default)
//   cargo bench --bench data_channel_buffer -p webrtc --no-default-features \
//     --features buf-handoff                                                           # post-commit
//
// Flamegraph:
//   cargo flamegraph --bench data_channel_buffer -p webrtc \
//     --features buf-copy -- --profile-time 10
//   cargo flamegraph --bench data_channel_buffer -p webrtc \
//     --no-default-features --features buf-handoff -- --profile-time 10

use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

const DATA_CHANNEL_BUFFER_SIZE: usize = u16::MAX as usize; // 65535, matches the constant in mod.rs

// ---------------------------------------------------------------------------
// Both implementations inline — mirrors data_channel/mod.rs package_message exactly.
// The bench file cannot access pub(crate) items, so we duplicate the logic here.
// ---------------------------------------------------------------------------

/// Pre-commit strategy: allocate a fresh Vec of `n` bytes and copy the valid portion.
/// Memory: one buffer of 65535 always resident + transient `n`-byte allocation per message.
/// CPU: one alloc + memcpy of `n` bytes per message.
#[cfg(feature = "buf-copy")]
fn package_message(buffer: &mut Vec<u8>, n: usize) -> Bytes {
    Bytes::from(buffer[..n].to_vec())
}

/// Post-commit strategy: hand off the old buffer and allocate a fresh 65535-byte one.
/// Memory: up to two 65535-byte buffers in flight (old held by Bytes + new for next read).
/// CPU: one 65535-byte alloc per message; zero copy of data.
///
/// WARN (Bug 1): `Vec::with_capacity` gives len=0. In the real read loop the next
///   `read_data_channel(&mut buffer)` call would get an empty slice and return Ok((0,_)),
///   triggering immediate channel close. The bench avoids this by reinitialising the buffer
///   after each call (matching what the pre-alloc intent should have been).
///
/// WARN (Bug 2): `Bytes::from(old)` where `old.len() == 65535` sends the full zeroed buffer,
///   not just the `n` valid bytes received.
#[cfg(feature = "buf-handoff")]
fn package_message(buffer: &mut Vec<u8>, _n: usize) -> Bytes {
    let new_buf = Vec::with_capacity(DATA_CHANNEL_BUFFER_SIZE);
    let old = std::mem::replace(buffer, new_buf);
    Bytes::from(old)
}

// ---------------------------------------------------------------------------
// Benchmark
// ---------------------------------------------------------------------------

fn bench_package_message(c: &mut Criterion) {
    // Three message sizes chosen to span the realistic WebRTC data channel range:
    //
    //     64 B  — typical signaling / control messages.  Copy cost is tiny relative to alloc
    //             overhead, so the two strategies should look similar here.
    //   1400 B  — typical data payload near Ethernet MTU.  Midpoint of the tradeoff.
    //  16384 B  — large blob (WebRTC spec allows up to DATA_CHANNEL_BUFFER_SIZE).  Copy cost
    //             dominates here, which is where buf-handoff would show its biggest advantage
    //             if the bug were fixed.
    let sizes: &[usize] = &[64, 1400, 16384];

    let mut group = c.benchmark_group("package_message");

    for &n in sizes {
        // Pre-fill the buffer with `n` bytes of realistic-looking data so that buf-copy
        // actually copies real content rather than measuring a trivially-predictable pattern.
        let payload: Vec<u8> = (0..n).map(|i| (i & 0xFF) as u8).collect();

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            // Initialise outside the iter loop so setup cost is not measured.
            let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
            buffer[..n].copy_from_slice(&payload);

            b.iter(|| {
                let result = package_message(black_box(&mut buffer), black_box(n));

                // For buf-handoff: buffer.len() is now 0 after mem::replace (Bug 1).
                // Reinitialise so the next iteration has a valid read buffer, which mirrors
                // what correct code would do (vec![0u8; DATA_CHANNEL_BUFFER_SIZE]).
                if buffer.len() < n {
                    buffer.resize(DATA_CHANNEL_BUFFER_SIZE, 0);
                    buffer[..n].copy_from_slice(&payload);
                }

                // Prevent the compiler from eliding the Bytes allocation.
                black_box(result)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_package_message);
criterion_main!(benches);
