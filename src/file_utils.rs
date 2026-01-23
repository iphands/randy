use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::{fs, str};

// Helper function to filter lines by starting strings
fn filter_lines_by_starts<'a>(
    lines: impl Iterator<Item = &'a str> + 'a,
    filters: &'a [&str],
) -> impl Iterator<Item = &'a str> + 'a {
    lines.filter(move |s| filters.iter().any(|filter| s.starts_with(filter)))
}

// Helper function for line reading with error handling
fn read_lines_with_error_handling<T>(
    mut reader: impl BufRead,
    line_end: usize,
    mut process_line: impl FnMut(&str) -> Option<T>,
) -> Result<Vec<T>, std::io::Error> {
    let mut lines = Vec::new();
    for _ in 0..line_end {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            _ => {
                if line.is_empty() {
                    break;
                }
                if let Some(result) = process_line(&line.trim()) {
                    lines.push(result);
                }
            }
        }
    }
    Ok(lines)
}

pub fn get_strings_from_path(path: &str, line_end: usize) -> Vec<String> {
    match try_strings_from_path(path, line_end) {
        Ok(v) => v,
        Err(e) => panic!("Unable to open / read {}: {}", &path, e),
    }
}

pub fn get_match_strings_from_path(path: &str, filters: &Vec<&str>) -> Vec<String> {
    match try_match_strings_from_path(path, &filters) {
        Ok(v) => v,
        Err(e) => panic!("Unable to open / read {}: {}", &path, e),
    }
}

#[inline(always)]
pub fn try_exact_match_strings_from_reader(
    reader: &mut BufReader<File>,
    filters: &Vec<&str>,
    hack: Option<fn(&i32, &str) -> bool>,
) -> Result<Vec<String>, std::io::Error> {
    let filter_count = filters.len() - 1;
    let mut count = 0;
    let mut lines_vec = Vec::new();
    let mut line_num = -1;
    let line = &mut String::new();

    loop {
        line.clear();
        match reader.read_line(line) {
            Ok(0) => return Ok(lines_vec),
            Err(e) => return Err(e),
            _ => {
                if line == "" {
                    return Ok(lines_vec);
                }
                line_num += 1;

                match hack {
                    Some(f) => match f(&line_num, &line) {
                        false => return Ok(lines_vec),
                        true => (),
                    },
                    None => (),
                };

                match filters.iter().try_for_each(|filter| {
                    if line.starts_with(filter) {
                        let l = line.trim().to_string();
                        lines_vec.push(l);
                        if count == filter_count {
                            return None;
                        }
                        count += 1;
                    }
                    return Some(());
                }) {
                    None => {
                        return Ok(lines_vec);
                    }
                    _ => (),
                };
            }
        }
    }
}

pub fn try_match_strings_from_file(file: &mut File, filters: &Vec<&str>) -> Result<Vec<String>, std::io::Error> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(filter_lines_by_starts(contents.lines(), filters)
        .map(|s| String::from(s))
        .collect())
}

pub fn try_match_strings_from_path(path: &str, filters: &Vec<&str>) -> Result<Vec<String>, std::io::Error> {
    let contents = fs::read_to_string(&path)?;
    Ok(filter_lines_by_starts(contents.lines(), filters)
        .map(|s| String::from(s))
        .collect())
}

#[inline(always)]
pub fn try_strings_from_reader(reader: &mut BufReader<File>, line_end: usize) -> Result<Vec<String>, std::io::Error> {
    reader.seek(SeekFrom::Start(0))?;
    read_lines_with_error_handling(reader, line_end, |line| {
        if line.is_empty() {
            None
        } else {
            Some(String::from(line))
        }
    })
}

#[inline(always)]
pub fn try_strings_from_path(path: &str, line_end: usize) -> Result<Vec<String>, std::io::Error> {
    let file = BufReader::new(File::open(path)?);
    read_lines_with_error_handling(
        file,
        line_end,
        |line| {
            if line.is_empty() {
                None
            } else {
                Some(String::from(line))
            }
        },
    )
}
