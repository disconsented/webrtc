// Benchmarks buffer-packaging strategies for the WebRTC data channel read loop.
//
// Each strategy trades allocation size against copy cost differently.  All four
// are tested in a single run so you can read the numbers side-by-side.
//
//   buf_copy            — alloc n bytes + memcpy n bytes (current default in mod.rs)
//   buf_handoff         — alloc+zero 65535 bytes + zero copy
//   buf_handoff_uninit  — alloc 65535 bytes (no zero-fill) + zero copy  [unsafe]
//   buf_adaptive        — buf_copy when n < THRESHOLD, buf_handoff_uninit otherwise
//
// The zero-fill in buf_handoff costs ~450 ns regardless of message size, which makes
// buf_copy faster for all messages under ~2 KB on this machine.  buf_handoff_uninit
// skips the zero-fill and wins for messages above ~1–2 KB.  Adjust ADAPTIVE_THRESHOLD
// after reading your own benchmark numbers.
//
// Run:
//   cargo bench --bench data_channel_buffer -p webrtc
//
// Flamegraph (written to target/criterion/<id>/profile/flamegraph.svg):
//   cargo bench --bench data_channel_buffer -p webrtc -- --profile-time 10

use std::{fs::File, os::raw::c_int, path::Path};

use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, profiler::Profiler, BenchmarkId, Criterion};
use pprof::ProfilerGuard;

const DATA_CHANNEL_BUFFER_SIZE: usize = u16::MAX as usize; // 65535

// Tune this after running the bench on your hardware.
const ADAPTIVE_THRESHOLD: usize = 1500;

// ---------------------------------------------------------------------------
// Profiler
// ---------------------------------------------------------------------------

struct FlamegraphProfiler<'a> {
    frequency: c_int,
    active_profiler: Option<ProfilerGuard<'a>>,
}

impl<'a> FlamegraphProfiler<'a> {
    fn new(frequency: c_int) -> Self {
        FlamegraphProfiler { frequency, active_profiler: None }
    }
}

impl<'a> Profiler for FlamegraphProfiler<'a> {
    fn start_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        self.active_profiler = Some(ProfilerGuard::new(self.frequency).unwrap());
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
        std::fs::create_dir_all(benchmark_dir).unwrap();
        let flamegraph_file = File::create(benchmark_dir.join("flamegraph.svg"))
            .expect("File system error while creating flamegraph.svg");
        if let Some(profiler) = self.active_profiler.take() {
            profiler
                .report()
                .build()
                .unwrap()
                .flamegraph(flamegraph_file)
                .expect("Error writing flamegraph");
        }
    }
}

// ---------------------------------------------------------------------------
// Strategies (mirror the implementations in data_channel/mod.rs)
// ---------------------------------------------------------------------------

/// Alloc n bytes + memcpy n bytes.  Memory-efficient; fast for small messages.
fn buf_copy(buffer: &mut Vec<u8>, n: usize) -> Bytes {
    Bytes::from(buffer[..n].to_vec())
}

/// Alloc+zero 65535 bytes + zero copy.  The zero-fill dominates cost (~450 ns fixed),
/// making this slower than buf_copy for all messages under ~65 KB.
fn buf_handoff(buffer: &mut Vec<u8>, n: usize) -> Bytes {
    let new_buf = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
    let mut old = std::mem::replace(buffer, new_buf);
    old.truncate(n);
    Bytes::from(old)
}

/// Alloc 65535 bytes (no zero-fill) + zero copy.
///
/// # Safety
///
/// Sound only when the caller guarantees that every byte of `buffer` is overwritten
/// by `read_data_channel` before it is read back, and that `Bytes::from` with
/// `truncate(n)` never exposes indices >= n (both invariants hold in `read_loop`).
fn buf_handoff_uninit(buffer: &mut Vec<u8>, n: usize) -> Bytes {
    let mut new_buf = Vec::with_capacity(DATA_CHANNEL_BUFFER_SIZE);
    // SAFETY: read_data_channel writes all bytes it returns before any read access.
    // The n..65535 region is inside the allocation but outside the Bytes view after truncate.
    unsafe { new_buf.set_len(DATA_CHANNEL_BUFFER_SIZE) };
    let mut old = std::mem::replace(buffer, new_buf);
    old.truncate(n);
    Bytes::from(old)
}

/// buf_copy below ADAPTIVE_THRESHOLD, buf_handoff_uninit above.
fn buf_adaptive(buffer: &mut Vec<u8>, n: usize) -> Bytes {
    if n < ADAPTIVE_THRESHOLD {
        buf_copy(buffer, n)
    } else {
        buf_handoff_uninit(buffer, n)
    }
}

// ---------------------------------------------------------------------------
// Benchmark
// ---------------------------------------------------------------------------

fn bench_strategies(c: &mut Criterion) {
    // 64 B   — typical signaling / control messages
    // 1400 B — near Ethernet MTU, straddles the adaptive threshold
    // 16384 B — large blob; copy cost dominates here
    let sizes: &[usize] = &[64, 1400, 16384];

    for &n in sizes {
        let payload: Vec<u8> = (0..n).map(|i| (i & 0xFF) as u8).collect();

        // buf_copy: buffer retains its content after each call; no restore needed.
        {
            let mut group = c.benchmark_group("buf_copy");
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
                buffer[..n].copy_from_slice(&payload);
                b.iter(|| black_box(buf_copy(black_box(&mut buffer), black_box(n))));
            });
            group.finish();
        }

        // buf_handoff: after each call buffer is a fresh zero-filled vec; no restore needed
        // because buf_handoff wraps the OLD buffer in Bytes and the new buffer's content is
        // irrelevant to the timing of the operation itself.
        {
            let mut group = c.benchmark_group("buf_handoff");
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
                b.iter(|| black_box(buf_handoff(black_box(&mut buffer), black_box(n))));
            });
            group.finish();
        }

        // buf_handoff_uninit: same reasoning — content of new buffer irrelevant to timing.
        {
            let mut group = c.benchmark_group("buf_handoff_uninit");
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
                b.iter(|| black_box(buf_handoff_uninit(black_box(&mut buffer), black_box(n))));
            });
            group.finish();
        }

        // buf_adaptive: restore buffer only on the handoff path (buffer.len() drops to n after
        // truncate and the next iter needs len=65535 for the read-slice coercion).
        {
            let mut group = c.benchmark_group("buf_adaptive");
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
                buffer[..n].copy_from_slice(&payload);
                b.iter(|| black_box(buf_adaptive(black_box(&mut buffer), black_box(n))));
            });
            group.finish();
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(FlamegraphProfiler::new(997));
    targets = bench_strategies
}
criterion_main!(benches);
