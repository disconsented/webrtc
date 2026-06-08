// Benchmarks comparing data channel buffer-packaging strategies.
//
// All strategies run in a single group so criterion generates comparison
// output (violin plots, % change vs upstream) in one pass.
//
//   upstream         — buf_copy: alloc n bytes + memcpy (the upstream default)
//   buf_handoff      — alloc+zero 65535 bytes + swap (upstream bug fixed: was Vec::with_capacity)
//   buf_handoff_uninit — alloc 65535 bytes (no zero-fill) + swap [unsafe]
//   buf_adaptive     — buf_copy below ADAPTIVE_THRESHOLD, buf_handoff_uninit above
//
// The zero-fill in buf_handoff costs ~450 ns regardless of message size, making
// buf_copy faster for messages under ~2 KB.  buf_handoff_uninit skips the fill
// and wins for messages above ~1–2 KB.  Adjust ADAPTIVE_THRESHOLD to match your
// hardware after reading the numbers.
//
// Run:
//   cargo bench --bench data_channel_buffer -p webrtc
//
// Flamegraph (written to target/criterion/<id>/profile/flamegraph.svg):
//   cargo bench --bench data_channel_buffer -p webrtc -- --profile-time 10

use std::{fs::File, os::raw::c_int, path::Path};

use bytes::Bytes;
use criterion::{
    black_box, criterion_group, criterion_main, profiler::Profiler, BenchmarkId, Criterion,
};
use pprof::ProfilerGuard;

const DATA_CHANNEL_BUFFER_SIZE: usize = u16::MAX as usize; // 65535
const ADAPTIVE_THRESHOLD: usize = 1500;

// ---------------------------------------------------------------------------
// Profiler
// ---------------------------------------------------------------------------

struct FlamegraphProfiler<'a> {
    frequency: c_int,
    active_profiler: Option<ProfilerGuard<'a>>,
}

impl<'a> FlamegraphProfiler<'a> {
    #[tracing::instrument(level = "debug", skip(frequency))]
    fn new(frequency: c_int) -> Self {
        FlamegraphProfiler { frequency, active_profiler: None }
    }
}

impl<'a> Profiler for FlamegraphProfiler<'a> {
    #[tracing::instrument(level = "debug", skip(self, _benchmark_id, _benchmark_dir))]
    fn start_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        self.active_profiler = Some(ProfilerGuard::new(self.frequency).unwrap());
    }

    #[tracing::instrument(level = "debug", skip(self, _benchmark_id, benchmark_dir))]
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

#[tracing::instrument(level = "debug", skip(buffer, n))]
/// Upstream default: alloc n bytes + memcpy n bytes.
fn buf_copy(buffer: &mut Vec<u8>, n: usize) -> Bytes {
    Bytes::from(buffer[..n].to_vec())
}

#[tracing::instrument(level = "debug", skip(buffer, n))]
/// Fixed handoff: alloc+zero 65535 bytes + swap.  Zero-fill (~450 ns) dominates for
/// small messages; only competitive against buf_copy above ~65 KB.
fn buf_handoff(buffer: &mut Vec<u8>, n: usize) -> Bytes {
    let new_buf = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
    let mut old = std::mem::replace(buffer, new_buf);
    old.truncate(n);
    Bytes::from(old)
}

#[tracing::instrument(level = "debug", skip(buffer, n))]
/// Uninit handoff: alloc 65535 bytes (no zero-fill) + swap.
///
/// # Safety
///
/// Sound only when the caller guarantees every byte of `buffer[..n]` is written by
/// `read_data_channel` before being read back, and that `truncate(n)` keeps the Bytes
/// view within the initialized [0..n] region (both invariants hold in `read_loop`).
fn buf_handoff_uninit(buffer: &mut Vec<u8>, n: usize) -> Bytes {
    let mut new_buf = Vec::with_capacity(DATA_CHANNEL_BUFFER_SIZE);
    unsafe { new_buf.set_len(DATA_CHANNEL_BUFFER_SIZE) };
    let mut old = std::mem::replace(buffer, new_buf);
    old.truncate(n);
    Bytes::from(old)
}

#[tracing::instrument(level = "debug", skip(buffer, n))]
/// Adaptive: buf_copy below ADAPTIVE_THRESHOLD, buf_handoff_uninit above.
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

#[tracing::instrument(level = "debug", skip(c))]
fn bench_strategies(c: &mut Criterion) {
    // 64 B   — typical signaling / control messages
    // 1400 B — near Ethernet MTU, straddles the adaptive threshold
    // 16384 B — large blob where copy cost dominates
    let sizes: &[usize] = &[64, 1400, 16384];

    let mut group = c.benchmark_group("package_message");

    for &n in sizes {
        let payload: Vec<u8> = (0..n).map(|i| (i & 0xFF) as u8).collect();

        // Upstream default: alloc + copy.  Buffer content is preserved across iters.
        group.bench_with_input(BenchmarkId::new("upstream", n), &n, |b, &n| {
            let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
            buffer[..n].copy_from_slice(&payload);
            b.iter(|| black_box(buf_copy(black_box(&mut buffer), black_box(n))));
        });

        // Fixed buf_handoff: swap + zero-fill on each iter.  New-buffer content irrelevant.
        group.bench_with_input(BenchmarkId::new("buf_handoff", n), &n, |b, &n| {
            let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
            b.iter(|| black_box(buf_handoff(black_box(&mut buffer), black_box(n))));
        });

        // Uninit handoff: swap without zero-fill.  New-buffer content irrelevant.
        group.bench_with_input(BenchmarkId::new("buf_handoff_uninit", n), &n, |b, &n| {
            let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
            b.iter(|| black_box(buf_handoff_uninit(black_box(&mut buffer), black_box(n))));
        });

        // Adaptive: copy path needs valid payload; handoff path ignores buffer content.
        group.bench_with_input(BenchmarkId::new("buf_adaptive", n), &n, |b, &n| {
            let mut buffer = vec![0u8; DATA_CHANNEL_BUFFER_SIZE];
            buffer[..n].copy_from_slice(&payload);
            b.iter(|| black_box(buf_adaptive(black_box(&mut buffer), black_box(n))));
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(FlamegraphProfiler::new(997));
    targets = bench_strategies
}
criterion_main!(benches);
