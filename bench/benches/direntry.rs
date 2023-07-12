use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::collections::{HashSet};

#[allow(dead_code)]
fn chars() -> HashSet<String> {
    let mut pids = HashSet::new();
    for dir_entry in fs::read_dir("/proc").unwrap() {
        let entry: fs::DirEntry = match dir_entry {
            Ok(r)  => r,
            Err(_) => continue,
        };

        let path = &entry.path().display().to_string();
        if path.chars().nth(6).unwrap().is_ascii_digit() {
            let pid = &path[6..];
            pids.insert(pid.to_string());
        }
    }

    pids
}

#[allow(dead_code)]
fn chars_skip() -> HashSet<String> {
    let mut pids = HashSet::new();
    for dir_entry in fs::read_dir("/proc").unwrap() {
        let entry: fs::DirEntry = match dir_entry {
            Ok(r)  => r,
            Err(_) => continue,
        };

        let path = String::from(entry.path().to_str().unwrap());
        if path.chars().nth(6).unwrap().is_ascii_digit() {
            let pid = &path[6..];
            pids.insert(pid.to_string());
        }
    }

    pids
}

#[allow(dead_code)]
fn bytes() -> HashSet<String> {
    let mut pids = HashSet::new();
    for dir_entry in fs::read_dir("/proc").unwrap() {
        let entry: fs::DirEntry = match dir_entry {
            Ok(r)  => r,
            Err(_) => continue,
        };

        let path = &entry.path().display().to_string();
        if path.as_bytes()[6].is_ascii_digit() {
            let pid = &path[6..];
            pids.insert(pid.to_string());
        }
    }

    pids
}

#[allow(dead_code)]
fn bytes_no_display() -> HashSet<String> {
    let mut pids = HashSet::new();
    for dir_entry in fs::read_dir("/proc").unwrap() {
        let entry: fs::DirEntry = match dir_entry {
            Ok(r)  => r,
            Err(_) => continue,
        };

        let path = String::from(entry.path().to_str().unwrap());
        if path.as_bytes()[6].is_ascii_digit() {
            let pid = &path[6..];
            pids.insert(pid.to_string());
        }
    }

    pids
}

#[allow(dead_code)]
fn bytes_foreach() -> HashSet<String> {
    let mut pids = HashSet::new();

    fs::read_dir("/proc").unwrap().for_each(|dir_entry| {
        let entry: fs::DirEntry = match dir_entry {
            Ok(r)  => r,
            Err(_) => return,
        };

        let path = &entry.path().display().to_string();
        if path.as_bytes()[6].is_ascii_digit() {
            let pid = &path[6..];
            pids.insert(pid.to_string());
        }
    });

    pids
}

fn filter_map() -> HashSet<String> {
    let mut pids = HashSet::new();

    fs::read_dir("/proc").unwrap()
        .filter_map(|dir_entry| {
            let entry: fs::DirEntry = match dir_entry {
                Ok(r)  => r,
                Err(_) => return None,
            };
            Some(String::from(entry.path().to_str().unwrap()))
        }).for_each(|path| {
            if path.as_bytes()[6].is_ascii_digit() {
                let pid = &path[6..];
                pids.insert(pid.to_string());
            }
        });

    pids
}


fn filter_map2() -> HashSet<String> {
    let mut pids = HashSet::new();

    fs::read_dir("/proc").unwrap()
        .filter_map(|dir_entry| {
            let entry: fs::DirEntry = match dir_entry {
                Ok(r)  => r,
                Err(_) => return None,
            };
            Some(String::from(entry.path().to_string_lossy()))
        }).for_each(|path| {
            if path.as_bytes()[6].is_ascii_digit() {
                let pid = &path[6..];
                pids.insert(pid.to_string());
            }
        });

    pids
}


fn filter_map3() -> HashSet<String> {
    let mut pids = HashSet::new();

    fs::read_dir("/proc").unwrap()
        .filter_map(|dir_entry| {
            let entry: fs::DirEntry = match dir_entry {
                Ok(r)  => r,
                Err(_) => return None,
            };
            Some(entry.path().into_os_string().into_string().unwrap())
        }).for_each(|path| {
            if path.as_bytes()[6].is_ascii_digit() {
                let pid = &path[6..];
                pids.insert(pid.to_string());
            }
        });

    pids
}

fn bytes_foreach_into_os_str() -> HashSet<String> {
    let mut pids = HashSet::new();

    fs::read_dir("/proc").unwrap().for_each(|dir_entry| {
        let entry: fs::DirEntry = match dir_entry {
            Ok(r)  => r,
            Err(_) => return,
        };

        let path = entry.path().into_os_string().into_string().unwrap();
        // let path = String::from(entry.path().to_str().unwrap());
        if path.as_bytes()[6].is_ascii_digit() {
            let pid = &path[6..];
            pids.insert(pid.to_string());
        }
    });

    pids
}

fn bytes_foreach_no_display() -> HashSet<String> {
    let mut pids = HashSet::new();

    fs::read_dir("/proc").unwrap().for_each(|dir_entry| {
        let entry: fs::DirEntry = match dir_entry {
            Ok(r)  => r,
            Err(_) => return,
        };

        // let path = entry.path().into_os_string().into_string().unwrap();
        let path = String::from(entry.path().to_str().unwrap());
        if path.as_bytes()[6].is_ascii_digit() {
            let pid = &path[6..];
            pids.insert(pid.to_string());
        }
    });

    pids
}

fn criterion_benchmark(c: &mut Criterion) {
    // println!("{:?}", bytes_foreach_skip());
    // println!("{:?}", filter_map());

    let mut group = c.benchmark_group("direntry");
    for i in 0..50 {
        group.bench_function(format!("filma3 {}", i), |b| b.iter(filter_map3));
        group.bench_function(format!("filma2 {}", i), |b| b.iter(filter_map2));
        group.bench_function(format!("filmap {}", i), |b| b.iter(filter_map));
        group.bench_function(format!("bytes3 {}", i), |b| b.iter(bytes_foreach_no_display));
        group.bench_function(format!("bytes4 {}", i), |b| b.iter(bytes_foreach_into_os_str));
        // group.bench_function(format!("bytes2 {}", i), |b| b.iter(|| bytes_foreach()));
        // group.bench_function(format!("bytes  {}", i), |b| b.iter(|| bytes()));
        // group.bench_function(format!("chars2 {}", i), |b| b.iter(|| chars_skip()));
        // group.bench_function(format!("chars  {}", i), |b| b.iter(|| chars()));
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
