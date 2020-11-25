use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[path = "../../src/macros.rs"]
mod macros;

fn tester(s: &str) -> String {
    format!("foo {}", s)
}

fn criterion_benchmark(c: &mut Criterion) {
    println!("raw   looks like: {}", tester(black_box("bar")));
    println!("macro looks like: {}", timings!("tester", tester, black_box("bar")));

    let mut group = c.benchmark_group("timings test");
    group.bench_function("raw", |b| b.iter(|| tester(black_box("bar"))));
    group.bench_function("macro", |b| b.iter(|| timings!("tester", tester, black_box("bar"))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
