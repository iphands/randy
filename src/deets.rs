#[cfg(not(feature = "timings"))]
use std::thread;
#[cfg(not(feature = "timings"))]
use std::time;

use libc::{c_char, c_int, c_ulong};
use lazy_static::lazy_static;
use yaml_rust::Yaml;

use std::{
    str, mem, slice,
    fs::{self, File},
    io::{BufReader, SeekFrom, Seek},
    sync::Mutex,
    ffi::CString,
    process::Command,
    collections::{HashMap, HashSet},
};

use crate::{
    file_utils::*, 
    split_to_strs, 
    timings, 
    split_spc_to_strs,
};

#[cfg(feature = "nvidia")]
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
#[cfg(feature = "nvidia")]
use nvml_wrapper::Nvml;

pub struct FileSystemUsage {
    pub used:  f64,
    pub total: f64,
    pub used_str: String,
    pub total_str: String,
    pub use_pct: String,
}
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
    pub mem_total: f64,
    pub mem_free: f64,
    pub net_dev: HashMap<String, (u64, u64)>,
    pub ps_info: Vec<PsInfo>,
    proc_stat: Vec<String>,
    sysinfo: libc::sysinfo,
    utsname: libc::utsname,
}

const LOAD_SHIFT_F32: f32 = (1 << libc::SI_LOAD_SHIFT) as f32;

// 1000000
#[cfg(not(feature = "timings"))]
const YIELD_TIME: time::Duration = time::Duration::from_nanos(1024);

type Buffers = (BufReader<File>, BufReader<File>);

lazy_static! {
    // this one should be separate from frame cache
    // it has to persist beyond a single frame
    static ref CPU_LOADS:      Mutex<HashMap<i32, CpuLoad>> = Mutex::new(HashMap::new());
    static ref PROC_LOAD_HIST: Mutex<HashMap<u32, (f64, f64)>> = Mutex::new(HashMap::new());
    static ref PROC_PID_FILES: Mutex<HashMap<String, BufReader<File>>> = Mutex::new(HashMap::new());
    static ref PROC_STAT_READERS: Mutex<HashMap<u32, BufReader<File>>> = Mutex::new(HashMap::new());
    static ref MOUNTS_READER:  Mutex<BufReader<File>> = Mutex::new(BufReader::new(File::open("/proc/mounts").unwrap()));
    static ref CPU_INFO_FILE:  Mutex<File> = Mutex::new(File::open("/proc/cpuinfo").unwrap());
    static ref BATTERY_CACHE:  Mutex<HashMap<String, Buffers>> = Mutex::new(HashMap::new());
    pub static ref CPU_COUNT: i32 = get_match_strings_from_path("/proc/cpuinfo", &vec!["processor"]).len() as i32;
    pub static ref CPU_COUNT_FLOAT: f64 = *CPU_COUNT as f64;
}

#[cfg(feature = "nvidia")]
lazy_static! {
    static ref NVML: Mutex<nvml_wrapper::Nvml> = Mutex::new(Nvml::init().unwrap());
}

fn get_hostname_from_utsname(n: [c_char; 65]) -> String {
    let hostname: &[u8] = unsafe{ slice::from_raw_parts(n.as_ptr() as *const u8, n.len()) };
    str_from_bytes(hostname.to_vec())
}

fn get_utsname() -> libc::utsname {
    let mut utsname: libc::utsname = unsafe { mem::zeroed() };
    unsafe { libc::uname(&mut utsname); };
    utsname
}

fn get_uname(r: [c_char; 65]) -> String {
    let release: &[u8] = unsafe{ slice::from_raw_parts(r.as_ptr() as *const u8, r.len()) };
    str_from_bytes(release.to_vec())
}

const SECONDS_IN_A_MINUTE: i32 = 60;
const SECONDS_IN_AN_HOUR: i32  = 60 * SECONDS_IN_A_MINUTE;
const SECONDS_IN_A_DAY: i32    = 24 * SECONDS_IN_AN_HOUR;

/// Convert number of seconds into hours, minutes and seconds
fn get_uptime_string(uptime: c_int) -> String {
    // extract days
    let days = uptime / SECONDS_IN_A_DAY;

    // extract hours
    let hour_seconds = uptime % SECONDS_IN_A_DAY;
    let hours = hour_seconds / SECONDS_IN_AN_HOUR;

    // extract minutes
    let minute_seconds = hour_seconds % SECONDS_IN_AN_HOUR;
    let minutes = minute_seconds / SECONDS_IN_A_MINUTE;

    // extract the remaining seconds
    let seconds = minute_seconds % SECONDS_IN_A_MINUTE;

    format!("{days}d {hours}h {minutes:02}m {seconds:02}s")
}

fn get_sysinfo() -> libc::sysinfo {
    let mut sysinfo: libc::sysinfo = unsafe { mem::zeroed() };
    unsafe { libc::sysinfo(&mut sysinfo); };
    sysinfo
}

fn get_load(loads: [c_ulong; 3]) -> String {
    let mut load_arr: [f32; 3] = [0.0, 0.0, 0.0];

    for i in 0..3 {
        load_arr[i] = (loads[i] as f32) / LOAD_SHIFT_F32;
    }

    format!("{:.2} {:.2} {:.2}", load_arr[0], load_arr[1], load_arr[2])
}

fn get_procs_count(proc_stat: &[String]) -> String {
    return match proc_stat.iter().find(|line| { line.starts_with("procs_running") }) {
        Some(r) => r.replace("procs_running ", ""),
        _ => panic!("Couldn't find running procs in /proc/stat"),
    };
}

fn get_ram_usage() -> (f64, f64)  {
    fn reduce(i: u64) -> f64 {
        (i as f64) / 1024.0 / 1024.0
    }

    fn get_item(i: usize, v: &[String]) -> u64 {
        return v[i]
            .split_ascii_whitespace().collect::<String>()
            .split(':').collect::<Vec<&str>>()[1]
            .replace("kB", "").parse().unwrap();
    }

    let meminfo = get_strings_from_path("/proc/meminfo", 3);
    let free  = reduce(get_item(2, &meminfo));
    let total = reduce(get_item(0, &meminfo));
    (free, total)
}

pub fn get_cpu_mhz() -> Vec<u16> {
    let mut file = CPU_INFO_FILE.lock().unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    try_match_strings_from_file(&mut file, &vec!["cpu MHz"]).unwrap()
        .into_iter()
        .map(|s| {
            split_to_strs!(s, ": ")[1].parse::<f32>().unwrap() as u16
        }).collect()
}

fn get_proc_stat() -> Vec<String> {
    get_match_strings_from_path("/proc/stat", &vec!["cpu", "proc"])
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
                        // applying the templates in main ui_update

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

fn get_cpu_temp_sys(val: Option<&str>) -> String {
    match fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        Ok(s) => {
            match s.trim().parse::<u32>() {
                Ok(i) => {
                    match val {
                        Some(s) => s.replace("{}", format!("{}", (i / 1000)).as_str()),
                        _ => format!("{}C", (i / 1000)),
                    }
                },
                Err(e) => e.to_string(),
            }
        },
        _ => "unknown".to_string(),
    }
}

fn get_ps_from_proc(counter: u64, mod_top: u64, mem_used: f64) -> Vec<PsInfo> {
    let mut procs = Vec::new();
    let cpu_loads_map  = &mut CPU_LOADS.lock().unwrap();
    let proc_files_map = &mut PROC_PID_FILES.lock().unwrap();
    let should_run_retain = counter % (mod_top * 5) == 0;

    fn _hack(line_num: &i32, line: &str) -> bool {
        if line_num == &0 {
            let first = line.as_bytes()[6];
            if first == 107 {
                if line.starts_with("Name:\tkworker") || line.starts_with("Name:\tksoftirqd") {
                    return false;
                }
            } else if first == 109 && line.starts_with("Name:\tmigration/") {
                return false;
            }
        }

        true
    }

    #[inline(always)]
    fn _do_cpu(path: &str, pid: &str, total_time: f64) -> f32 {
        let proc_loads_map = &mut PROC_LOAD_HIST.lock().unwrap();
        let readers_map = &mut PROC_STAT_READERS.lock().unwrap();
        let pid_u32   = pid.parse::<u32>().unwrap();

        if let std::collections::hash_map::Entry::Vacant(e) = readers_map.entry(pid_u32) {
            let p = &format!("{}/stat", &path);
            let tmp_reader = BufReader::new(match File::open(p) {
                Ok(f)  => f,
                Err(_) => return 0.0,
            });

            e.insert(tmp_reader);
        }

        let reader = readers_map.get_mut(&pid_u32).unwrap();
        match reader.seek(SeekFrom::Start(0)) {
            Ok(_)  => (),
            Err(_) => {
                readers_map.remove(&pid_u32);
                return 0.0;
            },
        };

        let stat_line = match try_strings_from_reader(reader, 1) {
            Ok(v)  => v,
            Err(_) => {
                readers_map.remove(&pid_u32);
                return 0.0;
            },
        };

        let stat_vec  = split_spc_to_strs!(stat_line[0]);

        let proc_time: f64 = stat_vec[13].parse::<f64>().unwrap() + stat_vec[14].parse::<f64>().unwrap();

        proc_loads_map.entry(pid_u32).or_insert((0.0, 0.0));

        let last = proc_loads_map.get(&pid_u32).unwrap();
        let util = 100.0 * (proc_time - last.0) / (total_time - last.1);

        proc_loads_map.insert(pid_u32, (proc_time, total_time));
        util as f32
    }

    let mut pids = HashSet::new();
    let match_vec = &vec!["Name", "VmRSS"];

    fs::read_dir("/proc").unwrap().for_each(|dir_entry| {
        #[cfg(not(feature = "timings"))]
        thread::sleep(YIELD_TIME);

        let entry: fs::DirEntry = match dir_entry {
            Ok(r)  => r,
            Err(_) => return,
        };

        let path = entry.path().into_os_string().into_string().unwrap();
        if path.as_bytes()[6].is_ascii_digit() {
            let pid = &path[6..];

            if should_run_retain {
                pids.insert(pid.to_string());
            }

            let status_lines = match proc_files_map.contains_key(pid) {
                true => {
                    let reader = proc_files_map.get_mut(pid).unwrap();
                    match reader.seek(SeekFrom::Start(0)) {
                        Ok(_)  => (),
                        Err(_) => {
                            PROC_STAT_READERS.lock().unwrap().remove(&pid.parse::<u32>().unwrap());
                            proc_files_map.remove(pid);
                            return
                        },
                    }

                    match try_exact_match_strings_from_reader(reader, match_vec, Some(_hack)) {
                        Ok(s)  => { s },
                        Err(_) => {
                            PROC_STAT_READERS.lock().unwrap().remove(&pid.parse::<u32>().unwrap());
                            proc_files_map.remove(pid);
                            return;
                        },
                    }
                },
                false => {
                    let mut file = match File::open(format!("{}/status", &path)) {
                        Ok(f)  => f,
                        Err(_) => return,
                    };

                    match try_match_strings_from_file(&mut file, match_vec) {
                        Ok(vec) => {
                            let reader = BufReader::new(file);
                            proc_files_map.insert(pid.to_string(), reader);
                            vec
                        },
                        Err(_) => return,
                    }
                },
            };

            if status_lines.len() != 2 { return; }

            let proc_used = status_lines[1][7..(status_lines[1].len() - 3)].trim().parse::<f64>();


            if let Ok(used) = proc_used {
                procs.push(PsInfo {
                    comm: String::from(&status_lines[0][6..]),
                    pid: String::from(pid),
                    cpu: _do_cpu(&path, pid, cpu_loads_map[&0].total as f64),
                    mem: (used / mem_used) as f32,
                });
            }
        }
    });

    if should_run_retain {
        PROC_STAT_READERS.lock().unwrap().retain(|i, _| { pids.contains(&i.to_string()) });
        proc_files_map.retain(|i, _| { pids.contains(i) });
    }

    procs
}

#[cfg(feature = "include_dead")]
fn get_ps() -> Vec<PsInfo> {
    let output = match Command::new("ps")
        .arg("--no-headers")
        .arg("--sort")
        .arg("-pcpu")
        .arg("ax")
        .arg("-eo")
        .arg("pid,pcpu,pmem,comm")
        .output() {
            Ok(o) => o,
            Err(e) => panic!("Error running ps!: {}", e)
        };

    let mut ps_info_vec = Vec::new();
    let out_str = String::from_utf8_lossy(&output.stdout);

    for line in out_str.lines() {
        let tmp = split_spc_to_strs!(line)
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

pub fn get_battery(path: &str) -> (bool, String) {
    let mut reader_map = BATTERY_CACHE.lock().unwrap();
    let path_string = String::from(path);

    if !reader_map.contains_key(path) {
        let cap_reader    = BufReader::new(File::open(format!("{}/capacity", path)).unwrap());
        let status_reader = BufReader::new(File::open(format!("{}/status", path)).unwrap());
        reader_map.insert(path_string, (cap_reader, status_reader));
    }

    let (cap_reader, status_reader) = reader_map.get_mut(path).unwrap();
    let capacity = &try_strings_from_reader(cap_reader, 1).unwrap()[0];

    let status   = match try_strings_from_reader(status_reader, 1).unwrap()[0].as_str() {
        "Discharging" => false,
        "Full"        => true,
        "Unknown"     => true,
        _             => true,
    };

    (status, String::from(capacity))
}

fn get_cpu_voltage_rpi() -> String {
    let output = match Command::new("vcgencmd").arg("measure_volts").arg("core").output() {
        Ok(o) => o,
        Err(e) => panic!("Error running vcgencmd to get volts: {}", e)
    };

    let out_str = String::from_utf8_lossy(&output.stdout);
    String::from(split_to_strs!(out_str.trim(), '=')[1])
}

fn get_cpu_speed_rpi() -> String {
    let output = match Command::new("vcgencmd").arg("measure_clock").arg("arm").output() {
        Ok(o) => o,
        Err(e) => panic!("Error running vcgencmd to get clock: {}", e)
    };

    let out_str = String::from_utf8_lossy(&output.stdout);
    let mhz_str = split_to_strs!(out_str.trim(), '=')[1];
    let mhz = mhz_str.parse::<u32>().unwrap() / 1000 / 1000;

    format!("{:04} MHz", mhz)
}

#[cfg(feature = "nvidia")]
pub fn nvidia_gpu_info(idx: u32) -> HashMap<&'static str, String> {
    let nvml = NVML.lock().unwrap();
    let device = nvml.device_by_index(idx).unwrap();

    let model = device.name().unwrap(); // GeForce on my system
    let power_limit = device.enforced_power_limit().unwrap(); // 275k milliwatts on my system
    let _encoder_util = device.encoder_utilization().unwrap(); // Currently 0 on my system; Not encoding anything
    let memory_info = device.memory_info().unwrap(); // Currently 1.63/6.37 GB used on my system
    let fan_speed = device.fan_speed(0).unwrap(); // Currently 17% on my system
    let temperature = device.temperature(TemperatureSensor::Gpu).unwrap();

    let gpu_info = [
        ("model", model),
        ("fan_speed", format!("{fan_speed}%")),
        ("temp", format!("{temperature}C")),
        ("power_limit", format!("{power_limit:?}")),
        ("memory_info", format!("{memory_info:?}GB")),
    ];

    HashMap::from(gpu_info)
}

pub fn do_func(item: &Yaml, frame_cache: &FrameCache) -> String {
    let func: &str = item["func"].as_str().unwrap();
    let val: Option<&str> = item["val"].as_str();

    let ret: String = match func {
        "hostname" =>    timings!(func, get_hostname_from_utsname, frame_cache.utsname.nodename as [c_char; 65]),
        "kernel" =>      timings!(func, get_uname, frame_cache.utsname.release as [c_char; 65]),
        "uptime" =>      timings!(func, get_uptime_string, frame_cache.sysinfo.uptime as c_int),
        "load" =>        timings!(func, get_load, frame_cache.sysinfo.loads as [c_ulong; 3]),
        "procs_count" => timings!(func, get_procs_count, &frame_cache.proc_stat),

        "ram_usage" => format!("{:.2}GB / {:.2}GB",
                               (frame_cache.mem_total - frame_cache.mem_free),
                               frame_cache.mem_total),
        "cpu_usage" => format!("{:.2}%", timings!(func, get_cpu_usage, -1)),

        "cpu_temp_sys" =>    timings!(func, get_cpu_temp_sys, val),
        "cpu_speed_rpi" =>   timings!(func, get_cpu_speed_rpi),
        "cpu_voltage_rpi" => timings!(func, get_cpu_voltage_rpi),

        #[cfg(feature = "sensors")]
        "sensor_info" => timings!("sensors", get_sensor_info,
            item["sensor_name"].as_str().unwrap(),
            item["label_name"].as_str().unwrap(),
            item["val"].as_str().unwrap(),
            item["whole"].as_bool().unwrap()
        ),

        _ => {
            println!("Unknown func: {}", func);
            return String::from("unimpl");
        },
    };

    ret
}

pub fn get_fs(keys: Vec<&str>) -> HashMap<String, FileSystemUsage> {
    let mut map: HashMap<String, FileSystemUsage> = HashMap::new();

    let reader = &mut MOUNTS_READER.lock().unwrap();
    let lines = try_strings_from_reader(reader, 1024).unwrap();

    // TODO we know the items to look for ahead of time
    // optimize this!

    let keys_total = keys.len();
    let mut found_count = 0;

    lines.iter().find(|line| {
        let tokens = split_spc_to_strs!(line);
        keys.iter().find(|path| {
            if tokens[1] == **path {
                let test = CString::new(**path).unwrap();
                let mut statvfs: libc::statvfs = unsafe { mem::zeroed() };
                unsafe { libc::statvfs(test.as_ptr(), &mut statvfs) };

		let free  = ((statvfs.f_bsize as f64 / 1024.0) * (statvfs.f_bfree as f64 / 1024.0)) / 1024.0;
                let mut total = ((statvfs.f_frsize as f64 / 1024.0) * (statvfs.f_blocks as f64 / 1024.0)) / 1024.0;
                let mut used  = total - free;
                let mut size_char = 'G';

		if total < 1.0 {
                    used *= 100.0;
                    total *= 100.0;
                    size_char = 'M';
                }

                map.insert(String::from(**path), FileSystemUsage {
                    used,
                    total,
                    used_str: format!("{:.2}{}", used, size_char),
                    total_str: format!("{:.2}{}", total, size_char),
                    use_pct: format!("{:.0}%", (used / total) * 100.0),
                });

                found_count += 1;
            }
            found_count == keys_total
        });
        found_count == keys_total
    });

    map
}

#[cfg(feature = "include_dead")]
pub fn get_fs_from_df(keys: Vec<&str>) -> HashMap<String, FileSystemUsage> {
    let output = match Command::new("df").arg("-h").output() {
        Ok(o) => o,
        Err(e) => panic!("Error running df -h!: {}", e)
    };

    let out_str = String::from_utf8_lossy(&output.stdout);
    let mut map: HashMap<String, FileSystemUsage> = HashMap::new();

    for row in out_str.lines().into_iter().map(|line| {
        line.split_whitespace().map(String::from).collect::<Vec<String>>()
    }).collect::<Vec<Vec<String>>>() {
        for key in keys.iter() {
            if key == &row[5] {
                map.insert(String::from(*key), FileSystemUsage {
                    used: row[2][0..row[2].len()-1].parse().unwrap(),
                    total: row[1][0..row[1].len()-1].parse().unwrap(),
                    used_str: String::from(&row[2]),
                    total_str: String::from(&row[1]),
                    use_pct: String::from(&row[4]),
                });
            }
        }
    }

    return map;
}

fn _do_top(counter: u64, mod_top: u64, do_top_bool: bool, mem_total: f64) -> Vec<PsInfo> {
    match do_top_bool {
        true => get_ps_from_proc(counter, mod_top, mem_total * 10000.0),
        false => Vec::new()
    }
}

pub fn get_frame_cache(counter: u64, mod_top: u64, do_top_bool: bool) -> FrameCache {
    let proc_stat = timings!("proc_stat", get_proc_stat);
    // Always warm this cache up!
    timings!("all_cpu", do_all_cpu_usage, &proc_stat);

    let mem = timings!("ram_usage", get_ram_usage);
    let ps_info = timings!("ps_info", _do_top, counter, mod_top, do_top_bool, mem.1);
    let sysinfo = timings!("sysinfo", get_sysinfo);
    let utsname = timings!("utsname", get_utsname);
    let net_dev = timings!("net_dev", get_net_dev);

    #[cfg(feature = "timings")]
    println!("Size of PROC_PID_FILES: {}", PROC_PID_FILES.lock().unwrap().len());
    #[cfg(feature = "timings")]
    println!("Size of PROC_STAT_READERS: {}\n", PROC_STAT_READERS.lock().unwrap().len());

    FrameCache {
        sysinfo,
        utsname,
        ps_info,
        proc_stat,
        mem_free:  mem.0,
        mem_total: mem.1,
        net_dev,
    }
}

fn get_net_dev() -> HashMap<String, (u64, u64)> {
    let lines = try_strings_from_path("/proc/net/dev", 1024).unwrap();
    let mut map: HashMap<String, (u64, u64)> = HashMap::new();

    lines.iter().skip(2).for_each(|line| {
        let tokens = split_spc_to_strs!(line);
        map.insert(String::from(&tokens[0][0..(tokens[0].len() - 1)]),
                   (tokens[9].parse::<u64>().unwrap(), tokens[1].parse::<u64>().unwrap()));
    });

    map
}

fn do_all_cpu_usage(proc_stat: &[String]) {
    let loads_map = &mut CPU_LOADS.lock().unwrap();

    for cpu_num in -1..*CPU_COUNT {
        loads_map.entry(cpu_num).or_insert(CpuLoad {
                idle:  0,
                total: 0,
                percent: 0.0,
            });

        let last_load = &loads_map[&cpu_num];

        let proc_stat_line_items: Vec<u64> = proc_stat[(cpu_num + 1) as usize]
            .split_ascii_whitespace()
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();

        let idle:  u64 = proc_stat_line_items[3];
        let total: u64 = proc_stat_line_items.iter().sum();

        let totals = total - last_load.total;
        let idles  = idle - last_load.idle;

        let mut percent = ((totals as f64 - (idles as f64)) / totals as f64) * 100.0;
        if percent.is_nan() { percent = 0.0 }

        loads_map.insert(cpu_num, CpuLoad {
            idle,
            total,
            percent,
        });
    }
}

pub fn get_cpu_usage(cpu_num: i32) -> f64 {
    let loads_map = CPU_LOADS.lock().unwrap();
    let last_load = &loads_map[&cpu_num];
    last_load.percent
}

#[inline(always)]
fn str_from_bytes(mut buffer: Vec<u8>) -> String {
    let end = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    buffer.resize(end, 0);
    String::from_utf8(buffer).unwrap()
}