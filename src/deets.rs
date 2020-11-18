use std::io::{BufRead, BufReader};
use std::fs::File;
use yaml_rust::{Yaml};
use std::sync::Mutex;
use std::{str, mem, slice, fs};
use libc::{c_char, c_int, c_ulong};
use std::collections::HashMap;
use std::process::Command;

struct CpuLoad {
    idle:  u64,
    total: u64,
    percent: f64,
}

pub struct PsInfo {
    pub pid: String,
    pub cpu: f32,
    pub mem: f32,
    pub comm: String,
}

pub struct FrameCache {
    sysinfo: libc::sysinfo,
    utsname: libc::utsname,
    pub ps_info: Vec<PsInfo>,
    proc_stat: Vec<String>,
    pub mem_total: f64,
    pub mem_free: f64
}

const LOAD_SHIFT_F32: f32 = (1 << libc::SI_LOAD_SHIFT) as f32;

lazy_static! {
    // this one should be separate from frame cache
    // it has to persist beyond a single frame
    static ref CPU_LOADS: Mutex<HashMap<i32 ,CpuLoad>> = Mutex::new(HashMap::new());
    pub static ref CPU_COUNT: i32 = get_file("/proc/cpuinfo".to_string(), "processor", 0).len() as i32;
    pub static ref CPU_COUNT_FLOAT: f64 = *CPU_COUNT as f64;
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

    for i in 0..3 {
        load_arr[i] = (loads[i] as f32) / LOAD_SHIFT_F32;
    }

    return format!("{:.2} {:.2} {:.2}", load_arr[0], load_arr[1], load_arr[2]);
}

fn get_procs(proc_stat: &Vec<String>) -> String {
    let mut running: Option<String> = None;

    for line in proc_stat {
        if line.starts_with("procs_running") {
            running = Some(line.replace("procs_running ", ""));
        }
    }

    match running {
        Some(r) => return r,
        _ => panic!("Couldn't find running procs in /proc/stat"),
    }
}

// fn get_ram_usage(totalram: u64, freeram: u64) -> String {
pub fn get_ram_usage() -> (f64, f64)  {
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
    // return format!("{:.2}GB / {:.2}GB", (total - free), total);
    return (free, total);
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

    let mut file = BufReader::new(File::open(&path).unwrap());
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

pub fn get_cpu_mhz() -> Vec<u16> {
    return get_file("/proc/cpuinfo".to_string(), "cpu MHz", 0)
        .into_iter()
        .map(|s| {
            return s.split(": ").collect::<Vec<&str>>()[1]
                .parse::<f32>().unwrap() as u16;
        }).collect();
}

fn get_proc_stat() -> Vec<String> {
    return get_file(String::from("/proc/stat"), "", 0);
}

fn do_all_cpu_usage(proc_stat: &Vec<String>) {
    let loads_map = &mut CPU_LOADS.lock().unwrap();

    for cpu_num in -1..*CPU_COUNT {
        if !loads_map.contains_key(&cpu_num) {
            loads_map.insert(cpu_num, CpuLoad {
                idle:  0,
                total: 0,
                percent: 0.0,
            });
        }

        let last_load = &loads_map[&cpu_num];

        let proc_stat_line_items: Vec<u64> = proc_stat[(cpu_num + 1) as usize]
            .split(' ')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();

        let idle:  u64 = proc_stat_line_items[3];
        let total: u64 = proc_stat_line_items.iter().fold(0, |a, b| a + b);

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
    let loads_map = CPU_LOADS.lock().unwrap();
    let last_load = &loads_map[&cpu_num];
    return last_load.percent;
}

#[cfg(feature = "sensors")]
fn get_sensor_info(sensor_name: &str, label_name: &str, val: &str, whole: bool) -> String {
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

                        if whole {
                            return String::from(val).replace("{}", format!("{:.0}", value).as_str());
                        }

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
                Ok(i) => format!("{}C", (i / 1000)),
                Err(e) => e.to_string(),
            }
        },
        _ => "unknown".to_string(),
    }
}

fn get_ps() -> Vec<PsInfo> {
    let output = match Command::new("ps")
        .arg("a")
        .arg("--no-headers")
        .arg("-o")
        .arg("pid,pcpu,pmem,comm")
        .output() {
            Ok(o) => o,
            Err(e) => panic!("Error running ps!: {}", e)
        };

    let mut ps_info_vec = Vec::new();
    let out_str = String::from_utf8_lossy(&output.stdout);
    for line in out_str.lines() {
        let tmp = line.split(" ")
            .collect::<Vec<&str>>()
            .into_iter()
            .filter(|s| s != &"")
            .collect::<Vec<&str>>();

        ps_info_vec.push(PsInfo {
            pid:  tmp[0].to_string(),
            cpu:  tmp[1].parse::<f32>().unwrap(),
            mem:  tmp[2].parse::<f32>().unwrap(),
            comm: tmp[3].to_string(),
        });
    }

    return ps_info_vec;
}

pub fn get_frame_cache() -> FrameCache {
    let proc_stat = get_proc_stat();

    // Always warm this cache up!
    do_all_cpu_usage(&proc_stat);

    let mem = get_ram_usage();

    return FrameCache {
        sysinfo:   get_sysinfo(),
        utsname:   get_utsname(),
        ps_info:   get_ps(),
        proc_stat: proc_stat,
        mem_free:  mem.0,
        mem_total: mem.1,
    };
}

fn get_cpu_voltage_rpi() -> String {
    let output = match Command::new("vcgencmd").arg("measure_volts").arg("core").output() {
        Ok(o) => o,
        Err(e) => panic!("Error running vcgencmd to get volts: {}", e)
    };

    let out_str = String::from_utf8_lossy(&output.stdout);
    return String::from(out_str.trim().split('=').collect::<Vec<&str>>()[1]);
}

fn get_cpu_speed_rpi() -> String {
    let output = match Command::new("vcgencmd").arg("measure_clock").arg("arm").output() {
        Ok(o) => o,
        Err(e) => panic!("Error running vcgencmd to get clock: {}", e)
    };

    let out_str = String::from_utf8_lossy(&output.stdout);
    let mhz_str = out_str.trim().split('=').collect::<Vec<&str>>()[1];
    let mhz = mhz_str.parse::<u32>().unwrap() / 1000 / 1000;

    return format!("{} MHz", mhz);
}


fn get_nvidia_gpu_temp() -> String {
    let output = match Command::new("nvidia-smi").arg("-q").arg("-d").arg("TEMPERATURE").output() {
        Ok(o) => o,
        Err(e) => panic!("Error running nvidia-smi -q -d TEMPERATURE: {}", e)
    };

    let out_str = String::from_utf8_lossy(&output.stdout);
    for line in out_str.lines() {
        if line.contains("GPU Current Temp") {
            return format!("{}C", line.split(": ").collect::<Vec<&str>>()[1].replace(" C", ""));
        }
    }

    return format!("unknown");
}

pub fn do_func(item: &Yaml, frame_cache: &FrameCache) -> String {
    let func: &str = item["func"].as_str().unwrap();

    let ret: String = match func {
        "hostname" => get_hostname_from_utsname(frame_cache.utsname.nodename as [c_char; 65]),
        "kernel" => get_uname(frame_cache.utsname.release as [c_char; 65]),
        "uptime" => get_uptime_string(frame_cache.sysinfo.uptime as c_int),
        "load" => get_load(frame_cache.sysinfo.loads as [c_ulong; 3]),
        "procs" => get_procs(&frame_cache.proc_stat),
        "ram_usage" => format!("{:.2}GB / {:.2}GB",
                               (frame_cache.mem_total - frame_cache.mem_free),
                               frame_cache.mem_total),
        "cpu_usage" => format!("{:.2}%", get_cpu_usage(-1)),
        "cpu_temp_sys" => get_cpu_temp_sys(),
        "cpu_speed_rpi" => get_cpu_speed_rpi(),
        "cpu_voltage_rpi" => get_cpu_voltage_rpi(),
        "nvidia_gpu_temp" => get_nvidia_gpu_temp(),

        #[cfg(feature = "sensors")]
        "sensor_info" => get_sensor_info(
            item["sensor_name"].as_str().unwrap(),
            item["label_name"].as_str().unwrap(),
            item["val"].as_str().unwrap(),
            item["whole"].as_bool().unwrap(),
        ),
        _ => {
            println!("Unkown func: {}", func);
            return String::from("unimpl");
        },
    };

    return ret;
}
