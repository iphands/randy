use std::{str, mem, slice};
use libc::{c_char, sysconf, _SC_HOST_NAME_MAX};

fn get_hostname() -> String {
    let hostname_max = unsafe { sysconf(_SC_HOST_NAME_MAX) };
    let mut name = vec![0 as u8; (hostname_max as usize) + 1];
    unsafe { libc::gethostname(name.as_mut_ptr() as *mut c_char, name.len()) };

    return str_from_bytes(name);
}

fn get_uname() -> String {
    let max = 65;
    let mut uname: libc::utsname = unsafe { mem::zeroed() };
    unsafe { libc::uname(&mut uname); };

    let release: &[u8] = unsafe{ slice::from_raw_parts(uname.release.as_ptr() as *const u8, uname.release.len()) };
    return str_from_bytes(release.to_vec());
}

fn str_from_bytes(mut buffer: Vec<u8>) -> String {
    let end = buffer.iter().position(|&b| b == 0).unwrap_or_else(|| buffer.len());
    buffer.resize(end, 0);

    return String::from_utf8(buffer).unwrap();
}

pub fn do_func(s: &str) -> String {
    get_uname();
    let ret: String = match s {
        "hostname" => get_hostname(),
        "kernel" => get_uname(),
        _ => {
            println!("Unkown func");
            return String::from("unimpl");
        },
    };

    return ret;
}
