use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn split_char(s: &str) -> Vec<String> {
    return s.split(' ').map(String::from).collect();
}

fn split_str(s: &str) -> Vec<String> {
    return s.split(' ').map(String::from).collect();
}

fn split_whitespace (s: &str) -> Vec<String> {
    return s.split_whitespace().map(String::from).collect::<Vec<String>>();
}

fn split_ascii_whitespace (s: &str) -> Vec<String> {
    return s.split_ascii_whitespace().map(String::from).collect::<Vec<String>>();
}

fn criterion_benchmark(c: &mut Criterion) {
    let s = black_box("This is a test  foo");
    println!("{:?}", split_char(s));
    println!("{:?}", split_str(s));
    println!("{:?}", split_whitespace(s));
    println!("{:?}", split_ascii_whitespace(s));

    let mut group = c.benchmark_group("direntry");
    for i in 0..50 {
        group.bench_function(format!("char     {}", i), |b| b.iter(|| split_char(black_box(s))));
        group.bench_function(format!("str      {}", i), |b| b.iter(|| split_str(black_box(s))));
        group.bench_function(format!("whitesp  {}", i), |b| b.iter(|| split_whitespace(black_box(s))));
        group.bench_function(format!("awhitesp {}", i), |b| b.iter(|| split_ascii_whitespace(black_box(s))));
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
