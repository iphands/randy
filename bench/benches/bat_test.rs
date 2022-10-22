use criterion::{criterion_group, criterion_main, Criterion};
use std::{str, fs};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

const BAT_FILE: &str = "/sys/devices/LNXSYSTM:00/LNXSYBUS:00/PNP0A08:00/device:19/PNP0C09:00/PNP0C0A:00/power_supply/BAT0/status";

fn get_byte(file: &mut File) -> bool {
    return match get_one_byte_from_file(file) {
        68 => false,
        70 => true,
        85 => true,
        _  => true,
    };
}

fn get_line(reader: &mut BufReader<File>) -> bool {
    let ret: &str = &get_one_line_from_reader(reader);
    return match ret {
        "Discharging" => false,
        "Full" => true,
        "Unknown" => true,
        _ => true,
    };
}

fn normal(reader: &mut BufReader<File>) -> bool {
    return match try_strings_from_reader(reader, 1).unwrap()[0].as_str() {
        "Discharging" => false,
        "Full" => true,
        "Unknown" => true,
        _ => true,
    };
}


pub fn get_one_byte_from_file(file: &mut File) -> u8 {
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut buffer = [0; 1];
    file.read(&mut buffer).unwrap();
    return buffer[0];
}

pub fn get_one_line_from_reader(reader: &mut BufReader<File>) -> String {
    reader.seek(SeekFrom::Start(0)).unwrap();
    let mut line = String::new();
    reader.read_line(&mut line);
    return line;
}

pub fn try_strings_from_reader(reader: &mut BufReader<File>, line_end: usize) -> Result<Vec<String>, std::io::Error> {
    reader.seek(SeekFrom::Start(0)).unwrap();
    let mut lines: Vec<String> = Vec::new();
    for _ in 0..line_end {
        let mut line = String::new();
        let e = match reader.read_line(&mut line) {
            Err(e) => Some(e),
            _ => None,
        };

        if e.is_some() { return Err(e.unwrap()); }
        if line == "" { break; }
        lines.push(String::from(line.trim()));
    }

    return Ok(lines);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut reader = BufReader::new(File::open(BAT_FILE).unwrap());
    let mut file   = File::open(BAT_FILE).unwrap();

    let mut group = c.benchmark_group("bat_test");

    println!("norm {}", normal(&mut reader));
    println!("line {}", get_line(&mut reader));
    println!("byte {}", get_byte(&mut file));

    for i in 0..5 {
        group.bench_function(format!("norm {}", i), |b| b.iter(|| normal(&mut reader)));
        group.bench_function(format!("byte {}", i), |b| b.iter(|| get_byte(&mut file)));
        group.bench_function(format!("line {}", i), |b| b.iter(|| get_line(&mut reader)));
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
