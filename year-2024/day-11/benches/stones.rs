use criterion::{criterion_group, criterion_main, Criterion};
use day_11::{memoized_run_stones, parse_input_str};
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    let stones = parse_input_str("9694820 93 54276 1304 314 664481 0 4").unwrap();

    c.bench_function("memoized_run_stones test_input 75", move |b| {
        b.iter(|| memoized_run_stones(black_box(&stones), black_box(75)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
