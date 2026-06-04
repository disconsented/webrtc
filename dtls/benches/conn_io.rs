// Benchmarks the two IO strategies introduced in commit b0897a1b for the DTLS connection layer.
//
// There are two independently toggleable pairs, selected by Cargo features:
//
//   Read path:  `dtls-buf-reader`  → BufReader (pre-commit)
//               (default)          → Cursor    (post-commit)
//
//   Write path: `dtls-buf-writer`  → BufWriter + unallocated vec (pre-commit)
//               (default)          → Vec::with_capacity(1200) + direct write (post-commit)
//
// Run commands:
//   cargo bench --bench conn_io -p dtls                                               # post-commit (defaults)
//   cargo bench --bench conn_io -p dtls --features dtls-buf-reader,dtls-buf-writer   # pre-commit
//
// Flamegraph:
//   cargo flamegraph --bench conn_io -p dtls -- --profile-time 10
//   cargo flamegraph --bench conn_io -p dtls \
//     --features dtls-buf-reader,dtls-buf-writer -- --profile-time 10

#[allow(unused_imports)] // BufWriter/Cursor are each unused under the opposite feature flag
use std::io::{BufWriter, Cursor};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dtls::application_data::ApplicationData;
use dtls::content::{Content, ContentType};
use dtls::record_layer::record_layer_header::{RecordLayerHeader, PROTOCOL_VERSION1_2};
use dtls::record_layer::RecordLayer;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_record_layer_header() -> RecordLayerHeader {
    RecordLayerHeader {
        content_type: ContentType::ApplicationData,
        protocol_version: PROTOCOL_VERSION1_2,
        epoch: 1,
        sequence_number: 42,
        content_len: 1200,
    }
}

fn make_record_layer(payload_len: usize) -> RecordLayer {
    let data = vec![0xABu8; payload_len];
    RecordLayer::new(
        PROTOCOL_VERSION1_2,
        1,
        Content::ApplicationData(ApplicationData { data }),
    )
}

/// Serialize `item` to bytes using the current write strategy.
///
/// - No feature (`dtls-buf-writer` off): `Vec::with_capacity(1200)` + direct marshal
/// - `dtls-buf-writer`: `vec![]` + `BufWriter` wrapper (pre-commit)
fn marshal_record(record: &RecordLayer) -> Vec<u8> {
    #[cfg(not(feature = "dtls-buf-writer"))]
    {
        // Post-commit: pre-allocate for typical MTU, write directly — avoids BufWriter overhead.
        let mut buf = Vec::with_capacity(1200);
        record.marshal(&mut buf).unwrap();
        buf
    }
    #[cfg(feature = "dtls-buf-writer")]
    {
        // Pre-commit: BufWriter adds an 8 KB internal heap buffer that is wasted when the
        // underlying writer is already an in-memory Vec.
        let mut buf = vec![];
        {
            let mut w = BufWriter::<&mut Vec<u8>>::new(buf.as_mut());
            record.marshal(&mut w).unwrap();
        }
        buf
    }
}

fn marshal_header(header: &RecordLayerHeader) -> Vec<u8> {
    #[cfg(not(feature = "dtls-buf-writer"))]
    {
        let mut buf = Vec::with_capacity(1200);
        header.marshal(&mut buf).unwrap();
        buf
    }
    #[cfg(feature = "dtls-buf-writer")]
    {
        let mut buf = vec![];
        {
            let mut w = BufWriter::<&mut Vec<u8>>::new(buf.as_mut());
            header.marshal(&mut w).unwrap();
        }
        buf
    }
}

/// Unmarshal a `RecordLayerHeader` from a byte slice using the current read strategy.
fn unmarshal_header(bytes: &[u8]) -> RecordLayerHeader {
    #[cfg(not(feature = "dtls-buf-reader"))]
    {
        // Post-commit: Cursor wraps an existing in-memory slice — no extra allocation or copy.
        let mut r = Cursor::new(bytes);
        RecordLayerHeader::unmarshal(&mut r).unwrap()
    }
    #[cfg(feature = "dtls-buf-reader")]
    {
        // Pre-commit: BufReader allocates its own 8 KB internal buffer even though the data is
        // already in RAM, adding pointless indirection.
        let mut r = std::io::BufReader::new(bytes);
        RecordLayerHeader::unmarshal(&mut r).unwrap()
    }
}

fn unmarshal_record(bytes: &[u8]) -> RecordLayer {
    #[cfg(not(feature = "dtls-buf-reader"))]
    {
        let mut r = Cursor::new(bytes);
        RecordLayer::unmarshal(&mut r).unwrap()
    }
    #[cfg(feature = "dtls-buf-reader")]
    {
        let mut r = std::io::BufReader::new(bytes);
        RecordLayer::unmarshal(&mut r).unwrap()
    }
}

// ---------------------------------------------------------------------------
// Benchmark groups
// ---------------------------------------------------------------------------

fn bench_write(c: &mut Criterion) {
    let header = make_record_layer_header();

    // write/record_layer_header — 13 bytes.
    // Represents the hot inner loop of `process_handshake_packet` where a header is serialized
    // for every handshake fragment before assembly.
    c.bench_function("write/record_layer_header", |b| {
        b.iter(|| marshal_header(black_box(&header)))
    });

    // write/record — realistic payload sizes.
    // Represents `write_packets`: the outermost write per outbound DTLS record.
    //   200  B — typical handshake message
    //   1200 B — near-MTU application data (the magic number in the pre-alloc comment)
    let mut group = c.benchmark_group("write/record");
    for payload_len in [200usize, 1200] {
        let record = make_record_layer(payload_len);
        group.bench_with_input(
            BenchmarkId::from_parameter(payload_len),
            &record,
            |b, r| b.iter(|| marshal_record(black_box(r))),
        );
    }
    group.finish();
}

fn bench_read(c: &mut Criterion) {
    // Build canonical byte payloads once so the bench only measures deserialization.

    // read/record_layer_header — 13 bytes.
    // Every inbound DTLS packet starts with a header unmarshal to determine content type.
    let header_bytes = {
        let h = make_record_layer_header();
        let mut buf = Vec::with_capacity(13);
        h.marshal(&mut buf).unwrap();
        buf
    };
    c.bench_function("read/record_layer_header", |b| {
        b.iter(|| unmarshal_header(black_box(&header_bytes)))
    });

    // read/record — realistic payload sizes.
    // Represents the full unmarshal path in `handle_incoming_packet` once the header epoch
    // has been validated and the packet is known to be non-handshake.
    //   200  B — typical alert or change-cipher-spec record
    //   1200 B — near-MTU application data record
    let mut group = c.benchmark_group("read/record");
    for payload_len in [200usize, 1200] {
        let record_bytes = {
            let rec = make_record_layer(payload_len);
            let mut buf = Vec::with_capacity(13 + payload_len);
            rec.marshal(&mut buf).unwrap();
            buf
        };
        group.bench_with_input(
            BenchmarkId::from_parameter(payload_len),
            &record_bytes,
            |b, bytes| b.iter(|| unmarshal_record(black_box(bytes))),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_write, bench_read);
criterion_main!(benches);
