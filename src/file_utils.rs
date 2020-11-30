use std::{str, fs};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::io::prelude::*;

pub fn get_strings_from_path(path: &str, line_end: usize) -> Vec<String> {
    match try_strings_from_path(path, line_end) {
        Ok(v)  => v,
        Err(e) => panic!("Unable to open / read {}: {}", &path, e),
    }
}

pub fn get_match_strings_from_path(path: &str, filters: &Vec<&str>) -> Vec<String> {
    match try_match_strings_from_path(path, &filters) {
        Ok(v)  => v,
        Err(e) => panic!("Unable to open / read {}: {}", &path, e),
    }
}

#[inline(always)]
pub fn try_exact_match_strings_from_reader(reader: &mut BufReader<File>, filters: &Vec<&str>, hack: Option<fn(&i32, &str) -> bool>) -> Result<Vec<String>, std::io::Error> {
    let filter_count = filters.len() - 1;
    let mut count = 0;
    let mut lines_vec = Vec::new();
    let mut line_num = -1;
    let line = &mut String::new();

    loop {
        line.clear();
        match reader.read_line(line) {
            Ok(0)  => return Ok(lines_vec),
            Err(e) => return Err(e),
            _ => {
                if line == "" { return Ok(lines_vec); }
                line_num += 1;

                match hack {
                    Some(f) => match f(&line_num, &line) {
                        false => return Ok(lines_vec),
                        true  => (),
                    },
                    None => (),
                };

                match filters.iter().try_for_each(|filter| {
                    if line.starts_with(filter) {
                        let l = line.trim().clone().to_string();
                        lines_vec.push(l);
                        if count == filter_count {
                            return None;
                        }
                        count += 1;
                    }
                    return Some(());
                }) {
                    None => { return Ok(lines_vec); },
                    _ => (),
                };
            },
        }
    }
}

pub fn try_match_strings_from_file(file: &mut File, filters: &Vec<&str>) -> Result<Vec<String>, std::io::Error> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    return Ok(contents.lines().filter(|s| {
        let mut ret = false;
        for filter in filters {
            ret = s.starts_with(filter);
            if ret { break; }
        }
        return ret;
    }).map(|s| String::from(s)).collect());
}

pub fn try_match_strings_from_path(path: &str, filters: &Vec<&str>) -> Result<Vec<String>, std::io::Error> {
    return match fs::read_to_string(&path) {
        Ok(s) => Ok(s.lines().filter(|s| {
            let mut ret = false;
            for filter in filters {
                ret = s.starts_with(filter);
                if ret { break; }
            }
            return ret;
        }).map(|s| String::from(s)).collect()),
        Err(e) => Err(e),
    };
}

#[inline(always)]
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

#[inline(always)]
pub fn try_strings_from_path(path: &str, line_end: usize) -> Result<Vec<String>, std::io::Error> {
    let mut file = BufReader::new(match File::open(&path) {
        Ok(f)  => f,
        Err(e) => return Err(e),
    });

    let mut lines: Vec<String> = Vec::new();
    for _ in 0..line_end {
        let mut line = String::new();
        let e = match file.read_line(&mut line) {
            Err(e) => Some(e),
            _ => None,
        };

        if e.is_some() { return Err(e.unwrap()); }
        if line == "" { break; }
        lines.push(String::from(line.trim()));
    }

    return Ok(lines);
}
