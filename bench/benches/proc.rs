#![allow(dead_code)]
// #![allow(unused_imports)]
#![allow(unused_must_use)]

use criterion::{criterion_group, criterion_main, Criterion};
use randy::file_utils;

use std::io::prelude::*;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

fn _hack(line_num: &i32, line: &str) -> bool {
    if line_num == &0 {
        let first = line.as_bytes()[6];
        if first == 107 {
            if line.starts_with("Name:\tkworker") || line.starts_with("Name:\tksoftirqd") {
                return false;
            }
        } else if first == 109 && line.starts_with("Name:\tmigration/") {
            return false;
        }
    }
    true
}

fn test_status(match_vec: &Vec<&str>, reader: &mut BufReader<File>) -> (String, f64) {
    reader.seek(SeekFrom::Start(0)).unwrap();
    let status_lines = file_utils::try_exact_match_strings_from_reader(reader, match_vec, Some(_hack)).unwrap();
    let used = status_lines[1][7..(status_lines[1].len() - 3)].trim().parse::<f64>().unwrap();
    let comm = String::from(&status_lines[0][6..]);

    (comm, used)
}

fn test_other(comm_path: &str, statm_path: &str) -> (String, f64) {
    let comm = &file_utils::get_strings_from_path(comm_path, 1)[0];
    let statm = &file_utils::get_strings_from_path(statm_path, 1)[0];
    let rss = (statm.split(' ').collect::<Vec<&str>>()[1].parse::<u64>().unwrap() * 4096) as f64;
    (String::from(comm), rss)
}

fn test_buff (comm_reader: &mut BufReader<File>, statm_reader: &mut BufReader<File>) -> (String, f64) {
    comm_reader.seek(SeekFrom::Start(0)).unwrap();
    statm_reader.seek(SeekFrom::Start(0)).unwrap();

    let comm = &mut String::new();
    comm_reader.read_line(comm);

    let statm = &mut String::new();
    statm_reader.read_line(statm);
    let rss = (statm.split(' ').collect::<Vec<&str>>()[1].parse::<u64>().unwrap() * 4096) as f64;
    (comm.trim().to_string(), rss)
}

fn criterion_benchmark(c: &mut Criterion) {
    let path = std::env::var("TPID").expect("You must set TPID env var to something like /proc/NNNN");
    let mut group = c.benchmark_group("proc");

    {
        let match_vec = vec!["Name", "VmRSS"];
        let file = File::open(format!("{}/status", &path)).unwrap();
        let mut reader = BufReader::new(file);
        group.bench_function("status", |b| b.iter(|| test_status(&match_vec, &mut reader)));
    }

    {
        let comm_path = &format!("{}/comm", &path);
        let statm_path = &format!("{}/statm", &path);
        group.bench_function("comm+statm", |b| b.iter(|| test_other (comm_path, statm_path)));
    }

    {
        let statm_file = File::open(format!("{}/statm", &path)).unwrap();
        let mut statm_reader = BufReader::new(statm_file);
        let comm_file = File::open(format!("{}/comm", &path)).unwrap();
        let mut comm_reader = BufReader::new(comm_file);
        group.bench_function("comm+statm cached", |b| b.iter(|| test_buff  (&mut comm_reader, &mut statm_reader)));
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
