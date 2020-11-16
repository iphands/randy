use std::io::{BufRead, BufReader};
use std::fs::File;
use yaml_rust::{Yaml};
use std::sync::Mutex;
use std::{str, mem, slice, fs};
use libc::{c_char, c_int, c_ulong};
use std::collections::HashMap;

struct CpuLoad {
    idle:  u64,
    total: u64,
    percent: f64,
}

lazy_static! {
    static ref CPU_LOADS: Mutex<HashMap<i32 ,CpuLoad>> = Mutex::new(HashMap::new());
    static ref CPU_COUNT: i32 = get_cpu_mhz().len() as i32;
}

fn get_hostname_from_utsname(n: [c_char; 65]) -> String {
    let hostname: &[u8] = unsafe{ slice::from_raw_parts(n.as_ptr() as *const u8, n.len()) };
    return str_from_bytes(hostname.to_vec());
}

fn get_utsname() -> libc::utsname {
    let mut utsname: libc::utsname = unsafe { mem::zeroed() };
    unsafe { libc::uname(&mut utsname); };
    return utsname;
}

fn get_uname(r: [c_char; 65]) -> String {
    let release: &[u8] = unsafe{ slice::from_raw_parts(r.as_ptr() as *const u8, r.len()) };
    return str_from_bytes(release.to_vec());
}

fn get_uptime_string(uptime: c_int) -> String {
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

fn get_load(loads: [c_ulong; 3]) -> String {
    let mut load_arr: [f32; 3] = [0.0, 0.0, 0.0];

    for (i, t) in loads.iter().enumerate() {
        let load: f32 = *t as f32;
        let info: f32 = load / 8.0 / 8000.0;
        load_arr[i] = info;
    }

    return String::from(format!("{:.2} {:.2} {:.2}", load_arr[0], load_arr[1], load_arr[2]));
}

fn get_procs(proc_stat: Vec<String>) -> String {
    let mut running: Option<String> = None;

    for line in proc_stat {
        if line.starts_with("procs_running") {
            running = Some(line.replace("procs_running ", ""));
        }
    }

    match running {
        Some(r) => return String::from(format!("{}", r)),
        _ => panic!("Couldn't find running procs in /proc/stat"),
    }
}

// fn get_ram_usage(totalram: u64, freeram: u64) -> String {
fn get_ram_usage() -> String {
    fn reduce(i: u64) -> f64 {
        return (i as f64) / 1024.0 / 1024.0;
    }

    fn get_item(i: usize, v: &Vec<String>) -> u64 {
        return v[i]
            .split(' ').collect::<String>()
            .split(':').collect::<Vec<&str>>()[1]
            .replace("kB", "").parse().unwrap();
    }

    let meminfo = get_file(String::from("/proc/meminfo"), "", 3);
    let free  = reduce(get_item(2, &meminfo));
    let total = reduce(get_item(0, &meminfo));
    return String::from(format!("{:.2}GB / {:.2}GB", (total - free), total));
}

fn get_file(path: String, filter: &str, line_end: usize) -> Vec<String> {
    if line_end == 0 {
        return match fs::read_to_string(&path) {
            Ok(s)  => s.lines()
                .filter(|s| {
                    if filter != "" {
                        return s.starts_with(filter);
                    }
                    return true;
                })
                .map(|s| String::from(s)).collect(),
            Err(_) => panic!("Unable to open / read {}", &path),
        };
    }

    let mut file = BufReader::new(File::open("/proc/meminfo").unwrap());
    let mut lines: Vec<String> = Vec::new();
    for _ in 0..line_end {
        let mut line = String::new();
        match file.read_line(&mut line) {
            Err(_) => panic!("Unable to open / read {}", &path),
            _ => (),
        };
        lines.push(String::from(line.replace('\n', "")));
    }

    return lines;
}

pub fn get_cpu_mhz() -> Vec<f64> {
    return get_file("/proc/cpuinfo".to_string(), "cpu MHz", 0)
        .into_iter()
        .map(|s| {
            return s.replace("cpu MHz		: ", "" )
                .parse::<f64>().unwrap();
        }).collect();
}

fn get_proc_stat() -> Vec<String> {
    return get_file(String::from("/proc/stat"), "", 0);
}

fn do_all_cpu_usage(proc_stat: &Vec<String>) {
    let mut loads_map = CPU_LOADS.lock().unwrap();

    for cpu_num in -1..*CPU_COUNT {
        if !loads_map.contains_key(&cpu_num) {
            loads_map.insert(cpu_num, CpuLoad {
                idle:  0,
                total: 0,
                percent: 0.0,
            });
        }

        let last_load = &loads_map[&cpu_num];

        let i: Vec<u64> = proc_stat[(cpu_num + 1) as usize].split(' ').filter_map(|s| s.parse::<u64>().ok()).collect();
        let idle:  u64 = i[3];
        let total: u64 = i.iter().fold(0, |a, b| a + b);

        let totals = total - last_load.total;
        let idles  = idle - last_load.idle;

        let mut percent = ((totals as f64 - (idles as f64)) / totals as f64) * 100.0;
        if percent.is_nan() { percent = 0.0 }

        loads_map.insert(cpu_num, CpuLoad {
            idle: idle,
            total: total,
            percent: percent,
        });
    }
}

pub fn get_cpu_usage(cpu_num: i32) -> f64 {
    let mut loads_map = CPU_LOADS.lock().unwrap();
    let last_load = &loads_map[&cpu_num];
    // return String::from(format!("{:.2}%", last_load.percent));
    return last_load.percent;
}

#[cfg(feature = "rand")]
fn get_sensor_info(sensor_name: &str, label_name: &str, val: &str) -> String {
    for chip in sensors::Sensors::new() {
        let name = chip.get_name().expect("name");
        if sensor_name == name {
            for feature in chip {
                let label = feature.get_label().expect("label");
                if label == label_name {
                    for subfeature in feature {
                        let value = subfeature.get_value().expect("value");
                        // TODO this beeegs for a proper templating solution
                        // I think I want to start returning the raw info and
                        // applying the tempates in main ui_update
                        return String::from(val).replace("{}", format!("{:.2}", value).as_str());
                    }
                }
            }
        }
    }

    return String::from("unknown");
}

fn get_cpu_temp_sys() -> String {
    match fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        Ok(s) => {
            match s.trim().parse::<u32>() {
                Ok(i) => String::from(format!("{}C", (i / 1000))),
                Err(e) => e.to_string(),
            }
        },
        _ => "unknown".to_string(),
    }
}

pub fn do_func(item: &Yaml) -> String {
    let func: &str = item["func"].as_str().unwrap();
    let sysinfo = get_sysinfo();
    let utsname = get_utsname();
    let proc_stat = get_proc_stat();

    let ret: String = match func {
        "hostname" => get_hostname_from_utsname(utsname.nodename as [c_char; 65]),
        "kernel" => get_uname(utsname.release as [c_char; 65]),
        "uptime" => get_uptime_string(sysinfo.uptime as c_int),
        "load" => get_load(sysinfo.loads as [c_ulong; 3]),
        "procs" => get_procs(proc_stat),
        "ram_usage" => get_ram_usage(),
        "cpu_usage" => {
            do_all_cpu_usage(&proc_stat);
            return String::from(format!("{:.2}%", get_cpu_usage(-1)));
        },
        "cpu_temp_sys" => get_cpu_temp_sys(),

        #[cfg(feature = "rand")]
        "sensor_info" => get_sensor_info(
            item["sensor_name"].as_str().unwrap(),
            item["label_name"].as_str().unwrap(),
            item["val"].as_str().unwrap(),
        ),
        _ => {
            println!("Unkown func: {}", func);
            return String::from("unimpl");
        },
    };

    return ret;
}
