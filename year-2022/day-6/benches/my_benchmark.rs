use criterion::{black_box, criterion_group, criterion_main, Criterion};
use day_6::{scan_for_start, old_scan_for_start};

fn roofline_unicode_respecting(input_str: &str) {
    for ch in input_str.chars() {
        black_box(ch);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let input_str = include_str!("../test_input2.txt");
    let input_str_only_solution = &input_str[..2204];

    c.bench_function("roofline_unicode_respecting test_input2.txt 14", |b| b.iter(|| roofline_unicode_respecting(black_box(input_str))));
    c.bench_function("old_scan_for_start test_input2.txt 14", |b| b.iter(|| old_scan_for_start(black_box(input_str), 14)));
    c.bench_function("scan_for_start test_input2.txt 14", |b| b.iter(|| scan_for_start(black_box(input_str), 14)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);