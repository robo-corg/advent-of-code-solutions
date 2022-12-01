use criterion::{black_box, criterion_group, criterion_main, Criterion};
use day_21::{play_part2, Game};


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("part 2", |b| b.iter(|| {

        play_part2(black_box(Game::new([6, 2])))
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);