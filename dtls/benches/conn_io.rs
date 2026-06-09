// Benchmarks comparing DTLS I/O strategies: upstream vs this branch.
//
// Each group runs both implementations side-by-side so criterion produces
// comparison output (violin plots, % change) in a single pass.
//
// Read path:   upstream → BufReader (allocates 8 KB internal buffer for already-RAM data)
//              branch   → Cursor   (zero-copy wrapper around the existing slice)
//
// Write path:  upstream → vec![] + BufWriter     (8 KB internal buffer, wasted)
//              branch   → Vec::with_capacity(1200) + direct write (no wrapper overhead)
//
// Run:
//   cargo bench --bench conn_io -p dtls
//
// Flamegraph:
//   cargo flamegraph --bench conn_io -p dtls -- --profile-time 10

use std::io::{BufReader, BufWriter, Cursor};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dtls::application_data::ApplicationData;
use dtls::content::{Content, ContentType};
use dtls::record_layer::record_layer_header::{RecordLayerHeader, PROTOCOL_VERSION1_2};
use dtls::record_layer::RecordLayer;

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

// ---------------------------------------------------------------------------
// Write strategies
// ---------------------------------------------------------------------------

fn marshal_header_upstream(header: &RecordLayerHeader) -> Vec<u8> {
    let mut buf = vec![];
    {
        let mut w = BufWriter::<&mut Vec<u8>>::new(buf.as_mut());
        header.marshal(&mut w).unwrap();
    }
    buf
}

fn marshal_header_branch(header: &RecordLayerHeader) -> Vec<u8> {
    let mut buf = Vec::with_capacity(13);
    header.marshal(&mut buf).unwrap();
    buf
}

fn marshal_record_upstream(record: &RecordLayer) -> Vec<u8> {
    let mut buf = vec![];
    {
        let mut w = BufWriter::<&mut Vec<u8>>::new(buf.as_mut());
        record.marshal(&mut w).unwrap();
    }
    buf
}

fn marshal_record_branch(record: &RecordLayer) -> Vec<u8> {
    // 1200 B matches the typical DTLS MTU; avoids BufWriter's 8 KB internal buffer.
    let mut buf = Vec::with_capacity(1200);
    record.marshal(&mut buf).unwrap();
    buf
}

// ---------------------------------------------------------------------------
// Read strategies
// ---------------------------------------------------------------------------

fn unmarshal_header_upstream(bytes: &[u8]) -> RecordLayerHeader {
    let mut r = BufReader::new(bytes);
    RecordLayerHeader::unmarshal(&mut r).unwrap()
}

fn unmarshal_header_branch(bytes: &[u8]) -> RecordLayerHeader {
    let mut r = Cursor::new(bytes);
    RecordLayerHeader::unmarshal(&mut r).unwrap()
}

fn unmarshal_record_upstream(bytes: &[u8]) -> RecordLayer {
    let mut r = BufReader::new(bytes);
    RecordLayer::unmarshal(&mut r).unwrap()
}

fn unmarshal_record_branch(bytes: &[u8]) -> RecordLayer {
    let mut r = Cursor::new(bytes);
    RecordLayer::unmarshal(&mut r).unwrap()
}

// ---------------------------------------------------------------------------
// Benchmark groups
// ---------------------------------------------------------------------------

fn bench_write(c: &mut Criterion) {
    let header = make_record_layer_header();

    // 13-byte header — the inner-loop hot path in process_handshake_packet.
    let mut group = c.benchmark_group("write/record_layer_header");
    group.bench_function("upstream", |b| {
        b.iter(|| marshal_header_upstream(black_box(&header)))
    });
    group.bench_function("branch", |b| {
        b.iter(|| marshal_header_branch(black_box(&header)))
    });
    group.finish();

    // Full record at realistic payload sizes — the outermost write per outbound DTLS record.
    //   200 B  — typical handshake message
    //   1200 B — near-MTU application data
    let mut group = c.benchmark_group("write/record");
    for payload_len in [200usize, 1200] {
        let record = make_record_layer(payload_len);
        group.bench_with_input(
            BenchmarkId::new("upstream", payload_len),
            &record,
            |b, r| b.iter(|| marshal_record_upstream(black_box(r))),
        );
        group.bench_with_input(
            BenchmarkId::new("branch", payload_len),
            &record,
            |b, r| b.iter(|| marshal_record_branch(black_box(r))),
        );
    }
    group.finish();
}

fn bench_read(c: &mut Criterion) {
    // Pre-build canonical byte payloads so the bench measures only deserialization.

    // 13-byte header — every inbound DTLS packet starts with a header unmarshal.
    let header_bytes = {
        let h = make_record_layer_header();
        let mut buf = Vec::with_capacity(13);
        h.marshal(&mut buf).unwrap();
        buf
    };

    let mut group = c.benchmark_group("read/record_layer_header");
    group.bench_function("upstream", |b| {
        b.iter(|| unmarshal_header_upstream(black_box(&header_bytes)))
    });
    group.bench_function("branch", |b| {
        b.iter(|| unmarshal_header_branch(black_box(&header_bytes)))
    });
    group.finish();

    // Full record unmarshal — handle_incoming_packet after epoch validation.
    //   200 B  — typical alert or change-cipher-spec record
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
            BenchmarkId::new("upstream", payload_len),
            &record_bytes,
            |b, bytes| b.iter(|| unmarshal_record_upstream(black_box(bytes))),
        );
        group.bench_with_input(
            BenchmarkId::new("branch", payload_len),
            &record_bytes,
            |b, bytes| b.iter(|| unmarshal_record_branch(black_box(bytes))),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_write, bench_read);
criterion_main!(benches);
