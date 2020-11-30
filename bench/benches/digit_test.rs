use criterion::{criterion_group, criterion_main, Criterion};

fn my_is_digit(byte: u8) -> bool {
    return match byte {
        48 => true,
        49 => true,
        50 => true,
        51 => true,
        52 => true,
        53 => true,
        54 => true,
        55 => true,
        56 => true,
        57 => true,
        _  => false,
    }
}

fn is_digit(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

fn criterion_benchmark(c: &mut Criterion) {
    let num_byte = "/proc/2135".bytes().nth(6).unwrap();
    let char_byte = "/proc/asdf".bytes().nth(6).unwrap();

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
