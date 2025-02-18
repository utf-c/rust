use criterion::{black_box, Criterion, Throughput};

pub fn compress(text: &str) -> Vec<u8> {
    match utf_c::compress(text) {
        Ok(r) => black_box(r),
        Err(e) => panic!("Compression failed: {:?}", e)
    }
}

pub fn decompress(bytes: &[u8]) -> Vec<u8> {
    match utf_c::decompress(bytes) {
        Ok(r) => black_box(r),
        Err(e) => panic!("Decompression failed: {:?}", e)
    }
}

fn benchmark(c: &mut Criterion) {
    let texts: [&str; 3] = [
        &"טקסט זה נדחס עם UTF-C ו-GZIP ולאחר מכן הושווה. טקסט זה תורגם עם Google Translate ואנו מקווים שהוא תורגם כהלכה, אך אין ערובה לכך",
        &"A".repeat(4 * 1024), // BASIC | 41
        &"𖽁".repeat(4 * 1024), // MIAO  | f0 96 bd 81
    ];

    for (idx, &text) in texts.iter().enumerate() {
        let group_name = format!("{}", idx);
        let mut group = c.benchmark_group(group_name);
        group.sample_size(250);

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function("compression", |b| b.iter(|| compress(text)));

        let compressed_bytes = compress(text);

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function("decompression", |b| b.iter(|| decompress(&compressed_bytes)));

        group.finish();

        println!("=============================================");
        let ratio = compressed_bytes.len() as f64 / text.len() as f64;
        let percentage = (1.0 - ratio) * 100.0;
        println!("Compression ratio: {:.4} ({:.2}%) [{} / {}]", ratio, percentage, compressed_bytes.len(), text.len());
        println!("=============================================");
    }
}

criterion::criterion_group!(benches, benchmark);
criterion::criterion_main!(benches);