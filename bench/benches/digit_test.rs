use criterion::{criterion_group, criterion_main, Criterion};

fn my_is_digit(byte: u8) -> bool {
    matches!(byte, 48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57)
}

fn is_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}

fn criterion_benchmark(c: &mut Criterion) {
    let num_byte = "/proc/2135".as_bytes()[6];
    let char_byte = "/proc/asdf".as_bytes()[6];

    let mut group = c.benchmark_group("direntry");
    for i in 0..50 {
        group.bench_function(format!("theirs {}", i), |b| b.iter(|| is_digit(num_byte)));
        group.bench_function(format!("mines  {}", i), |b| b.iter(|| my_is_digit(num_byte)));

        group.bench_function(format!("theirs! {}", i), |b| b.iter(|| is_digit(char_byte)));
        group.bench_function(format!("mines!  {}", i), |b| b.iter(|| my_is_digit(char_byte)));
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
