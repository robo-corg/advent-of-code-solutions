use criterion::{black_box, criterion_group, criterion_main, Criterion};
use day_6::{scan_for_start, old_scan_for_start};

fn criterion_benchmark(c: &mut Criterion) {
    let input_str = include_str!("../test_input2.txt");

    c.bench_function("old_scan_for_start test_input2.txt 14", |b| b.iter(|| old_scan_for_start(black_box(input_str), 14)));
    c.bench_function("scan_for_start test_input2.txt 14", |b| b.iter(|| scan_for_start(black_box(input_str), 14)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);