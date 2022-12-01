use std::io::Cursor;
use day_7::{brute_force_find_best, find_with_local_minima, parse_input, Input};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rand::{Rng, SeedableRng};

fn get_test_input() -> Input {
    let test_data_str = include_str!("../test_input.txt");

    let test_data_reader = Cursor::new(test_data_str.to_owned());

    parse_input(test_data_reader)
}


fn get_problem_input() -> Input {
    let test_data_str = include_str!("../input.txt");

    let test_data_reader = Cursor::new(test_data_str.to_owned());

    parse_input(test_data_reader)
}


fn test_input(c: &mut Criterion) {
    let test_input = get_test_input();

    c.bench_function("test_input_brute_force", |b| b.iter(|| brute_force_find_best(black_box(&test_input))));
    c.bench_function("test_input_local_minima", |b| b.iter(|| find_with_local_minima(black_box(&test_input))));
}

fn full_problem_input(c: &mut Criterion) {
    let test_input = get_problem_input();

    c.bench_function("full_input_brute_force", |b| b.iter(|| brute_force_find_best(black_box(&test_input))));
    c.bench_function("full_input_local_minima", |b| b.iter(|| find_with_local_minima(black_box(&test_input))));
}


fn synthetic_large(c: &mut Criterion) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x84dd8c6ffde769ae);

    let test_input: Vec<i32> = (0..100000).map(|_| rng.gen_range(0..10000)).collect();

    let mut group = c.benchmark_group("synthetic_brute_force");
    for size in [100usize, 500, 1000, 5000, 10000, 100000].iter() {
        //group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| brute_force_find_best(&test_input[..size]));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("synthetic_large_local_minima");
    for size in [100usize, 500, 1000, 5000, 10000, 100000].iter() {
        //group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| find_with_local_minima(&test_input[..size]));
        });
    }
    group.finish();
}


criterion_group!(benches, synthetic_large, test_input, full_problem_input);
criterion_main!(benches);

