use std::{str, mem, slice};
use libc::{c_char, sysconf, _SC_HOST_NAME_MAX};

fn get_hostname_from_utsname(n: [i8; 65]) -> String {
    let hostname: &[u8] = unsafe{ slice::from_raw_parts(n.as_ptr() as *const u8, n.len()) };
    return str_from_bytes(hostname.to_vec());
}

#[allow(dead_code)]
fn get_hostname() -> String {
    let hostname_max = unsafe { sysconf(_SC_HOST_NAME_MAX) };

    let mut name = vec![0 as u8; (hostname_max as usize) + 1];
    unsafe { libc::gethostname(name.as_mut_ptr() as *mut c_char, name.len()) };

    let mut domain = vec![0 as u8; (hostname_max as usize) + 1];
    unsafe { libc::getdomainname(domain.as_mut_ptr() as *mut c_char, domain.len()) };

    println!("{}", str_from_bytes(domain));
    return str_from_bytes(name);
}

fn get_utsname() -> libc::utsname {
    let mut utsname: libc::utsname = unsafe { mem::zeroed() };
    unsafe { libc::uname(&mut utsname); };
    return utsname;
}

fn get_uname(r: [i8; 65]) -> String {
    let release: &[u8] = unsafe{ slice::from_raw_parts(r.as_ptr() as *const u8, r.len()) };
    return str_from_bytes(release.to_vec());
}

fn get_uptime_string(uptime: i64) -> String {
    let d = uptime / 60 / 60 / 24;
    let h = (uptime / 60 / 60) - (d * 24);
    let m = (uptime / 60) - (h * 60) - ((d * 24) * 60);
    let s = (uptime) - ((d * 24) * 60 * 60) - (h * 60 * 60) - (m * 60);

    return format!("{}d {}h {:02}m {:02}s", d, h, m, s);
}

fn get_sysinfo() -> libc::sysinfo {
    let mut sysinfo: libc::sysinfo = unsafe { mem::zeroed() };
    unsafe { libc::sysinfo(&mut sysinfo); };
    return sysinfo;
}

fn str_from_bytes(mut buffer: Vec<u8>) -> String {
    let end = buffer.iter().position(|&b| b == 0).unwrap_or_else(|| buffer.len());
    buffer.resize(end, 0);
    return String::from_utf8(buffer).unwrap();
}

fn get_load(loads: [u64; 3]) -> String {
    let mut load_arr: [f32; 3] = [0.0, 0.0, 0.0];

    for (i, t) in loads.iter().enumerate() {
        let load: f32 = *t as f32;
        let info: f32 = load / 8.0 / 8000.0;
        load_arr[i] = info;
    }

    return String::from(format!("{:.2} {:.2} {:.2}", load_arr[0], load_arr[1], load_arr[2]));
}

fn get_procs(procs: u16) -> String {
    return String::from(format!("{}", procs));
}

fn get_ram_usage(totalram: u64, freeram: u64) -> String {
    let free =  (freeram as f64)  / 1024.0 / 1024.0 / 1024.0;
    let total = (totalram as f64) / 1024.0 / 1024.0 / 1024.0;
    return String::from(format!("{:.2}GB / {:.2}GB", free, total));
}

pub fn do_func(s: &str) -> String {
    let sysinfo = get_sysinfo();
    let utsname = get_utsname();

    let ret: String = match s {
        "hostname" => get_hostname_from_utsname(utsname.nodename),
        "kernel" => get_uname(utsname.release),
        "uptime" => get_uptime_string(sysinfo.uptime),
        "load" => get_load(sysinfo.loads),
        "procs" => get_procs(sysinfo.procs),
        "ram_usage" => get_ram_usage(sysinfo.totalram, sysinfo.freeram),
        _ => {
            println!("Unkown func: {}", s);
            return String::from("unimpl");
        },
    };

    return ret;
}
