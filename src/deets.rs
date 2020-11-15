use std::sync::RwLock;
use std::{str, mem, slice, fs};
use libc::{c_char, sysconf, _SC_HOST_NAME_MAX};

lazy_static! {
    static ref LAST_IDLE:  RwLock<u64> = RwLock::new(0);
    static ref LAST_TOTAL: RwLock<u64> = RwLock::new(0);
}

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

fn get_procs(procs: u16, proc_stat: Vec<String>) -> String {
    let mut running: Option<String> = None;

    for line in proc_stat {
        if line.starts_with("procs_running") {
            running = Some(line.replace("procs_running ", ""));
        }
    }

    match running {
        Some(r) => return String::from(format!("{} Running: {}", procs, r)),
        _ => return String::from(format!("{}", procs)),
    }
}

// fn get_ram_usage(totalram: u64, freeram: u64) -> String {
fn get_ram_usage(sysinfo: libc::sysinfo) -> String {
    fn reduce(i: u64) -> f64 {
        return (i as f64) / 1024.0 / 1024.0 / 1024.0;
    }

    let free  = reduce(sysinfo.freeram);
    let total = reduce(sysinfo.totalram);
    return String::from(format!("{:.2}GB / {:.2}GB", (total - free), total));
}

fn get_proc_stat() -> Vec<String> {
    return match fs::read_to_string("/proc/stat") {
        Ok(s)  => s.lines().map(|s| String::from(s)).collect(),
        Err(_) => panic!("fdsaf"),
    };
}

fn get_cpu_usage(proc_stat: Vec<String>) -> String {
    let i: Vec<u64> = proc_stat[0].split(' ').filter_map(|s| s.parse::<u64>().ok()).collect();

    let idle:  u64 = i[3];
    let total: u64 = i.iter().fold(0, |a, b| a + b);

    let totals = total - *LAST_TOTAL.read().unwrap();
    let idles  = idle -  *LAST_IDLE.read().unwrap();

    let percent = ((totals as f64 - (idles as f64)) / totals as f64) * 100.0;

    *LAST_IDLE.write().unwrap() = idle;
    *LAST_TOTAL.write().unwrap() = total;

    return String::from(format!("{:.2}%", percent));
}

pub fn do_func(s: &str) -> String {
    let sysinfo = get_sysinfo();
    let utsname = get_utsname();
    let proc_stat = get_proc_stat();

    let ret: String = match s {
        "hostname" => get_hostname_from_utsname(utsname.nodename),
        "kernel" => get_uname(utsname.release),
        "uptime" => get_uptime_string(sysinfo.uptime),
        "load" => get_load(sysinfo.loads),
        "procs" => get_procs(sysinfo.procs, proc_stat),
        "ram_usage" => get_ram_usage(sysinfo),
        "cpu_usage" => get_cpu_usage(proc_stat),
        _ => {
            println!("Unkown func: {}", s);
            return String::from("unimpl");
        },
    };

    return ret;
}
