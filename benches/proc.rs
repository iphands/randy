#[path = "../src/file_utils.rs"]
mod file_utils;

use std::io::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

const PID_DIR: &str = "/proc/3856796";

fn _hack(line_num: &i32, line: &str) -> bool {
    if line_num == &0 {
        let first = line.bytes().nth(6).unwrap();
        if first == 107 {
            if line.starts_with("Name:\tkworker") || line.starts_with("Name:\tksoftirqd") {
                return false;
            }
        } else if first == 109 {
            if line.starts_with("Name:\tmigration/") {
                return false;
            }
        }
    }
    return true;
}

fn test_status(match_vec: &Vec<&str>, mut reader: &mut BufReader<File>) -> (String, f64) {
    reader.seek(SeekFrom::Start(0)).unwrap();
    let status_lines = file_utils::try_exact_match_strings_from_reader(&mut reader, &match_vec, Some(_hack)).unwrap();
    let used = status_lines[1][7..(status_lines[1].len() - 3)].trim().parse::<f64>().unwrap();
    let comm = String::from(&status_lines[0][6..]);

    return (comm, used);
}

// fn test_other(comm_reader: &mut BufReader<File>, statm_reader: &mut BufReader<File>) -> (String, f64) {
fn test_other(comm_path: &str, statm_path: &str) -> (String, f64) {
    let comm = &file_utils::get_strings_from_path(comm_path, 1)[0];
    let statm = &file_utils::get_strings_from_path(statm_path, 1)[0];
    let rss = (statm.split(' ').collect::<Vec<&str>>()[1].parse::<u64>().unwrap() * 4096) as f64;
    return (String::from(comm), rss);
}

fn test_buff (comm_reader: &mut BufReader<File>, statm_reader: &mut BufReader<File>) -> (String, f64) {
    comm_reader.seek(SeekFrom::Start(0)).unwrap();
    statm_reader.seek(SeekFrom::Start(0)).unwrap();

    let comm = &mut String::new();
    comm_reader.read_line(comm);

    let statm = &mut String::new();
    statm_reader.read_line(statm);
    let rss = (statm.split(' ').collect::<Vec<&str>>()[1].parse::<u64>().unwrap() * 4096) as f64;
    return (comm.trim().to_string(), rss);
}

fn criterion_benchmark(c: &mut Criterion) {
    {
        let match_vec = vec!["Name", "VmRSS"];
        let mut file = File::open(&format!("{}/status", &PID_DIR)).unwrap();
        let mut reader = BufReader::new(file);
        c.bench_function("status", |b| b.iter(|| test_status(&match_vec, &mut reader)));
    }

    {
        let comm_path = &format!("{}/comm", &PID_DIR);
        let statm_path = &format!("{}/statm", &PID_DIR);
        c.bench_function("other ", |b| b.iter(|| test_other (&comm_path, &statm_path)));
    }

    {
        let mut statm_file = File::open(&format!("{}/statm", &PID_DIR)).unwrap();
        let mut statm_reader = BufReader::new(statm_file);
        let mut comm_file = File::open(&format!("{}/comm", &PID_DIR)).unwrap();
        let mut comm_reader = BufReader::new(comm_file);
        c.bench_function("otherb", |b| b.iter(|| test_buff  (&mut comm_reader, &mut statm_reader)));
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
