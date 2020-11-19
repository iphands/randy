use std::{str, fs};
use std::fs::File;
use std::io::{BufRead, BufReader};
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
        lines.push(String::from(line.trim()));
    }

    return Ok(lines);
}