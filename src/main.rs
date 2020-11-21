#[macro_use]
extern crate lazy_static;
extern crate gio;
extern crate gtk;
extern crate yaml_rust;

mod deets;
mod file_utils;

use gio::prelude::*;
use gtk::prelude::*;

use std::fs;
use std::collections::HashMap;
use std::env::args;
use std::sync::Mutex;

use yaml_rust::{YamlLoader, Yaml};

const SPACING: i32 = 5;

struct Cpu {
    mhz: gtk::Label,
    progress: gtk::ProgressBar,
    pct_label: gtk::Label,
}

struct TopRow {
    name: gtk::Label,
    pid: gtk::Label,
    pct: gtk::Label,
}

lazy_static! {
    static ref FRAME_COUNT: Mutex<u64> = Mutex::new(0);
}

fn get_css(conf: &Yaml) -> String {
    let css: String = String::from(include_str!("styles/app.css"));
    return css
        .replace("{ background_color }", conf["color_background"].as_str().unwrap_or("#000"))
        .replace("{ color }", conf["color_text"].as_str().unwrap_or("#fff"))
        .replace("{ label_color }", conf["color_label"].as_str().unwrap_or("#eee"))
        .replace("{ bar_color }", conf["color_bar"].as_str().unwrap_or("#fff"))
        .replace("{ font_size }", conf["font_size"].as_str().unwrap_or("large"));
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    let s: &str = &get_file();
    let config = &get_config(s)[0];

    //Add custom CSS

    let css: &str = &get_css(&config["settings"]);

    let screen = window.get_screen().unwrap();
    let provider = gtk::CssProvider::new();
    provider.load_from_data(css.as_bytes()).expect("Failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(&screen, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    window.set_title("Randy");
    window.set_decorated(config["settings"]["decoration"].as_bool().unwrap_or(false));
    window.set_resizable(config["settings"]["resizable"].as_bool().unwrap_or(false));
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(375, -1);
    window.set_skip_taskbar_hint(config["settings"]["skip_taskbar"].as_bool().unwrap_or(true));

    if !config["settings"]["xpos"].is_badvalue() &&
        !config["settings"]["ypos"].is_badvalue() {
            window.move_(
                config["settings"]["xpos"].as_i64().unwrap() as i32,
                config["settings"]["ypos"].as_i64().unwrap() as i32,
            );
        }

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    vbox.get_style_context().add_class("container");

    let mut values: HashMap<yaml_rust::Yaml, (gtk::Label, Option<gtk::ProgressBar>)> = HashMap::new();
    let mut cpus: Vec<Cpu>    = Vec::new();
    let mut top_mems: Vec<TopRow> = Vec::new();
    let mut top_cpus: Vec<TopRow> = Vec::new();
    let mut stash_fs: HashMap<String, (gtk::Label, gtk::ProgressBar)> = HashMap::new();

    init_ui(&mut values, &mut cpus, &mut top_mems, &mut top_cpus, &mut stash_fs, &vbox, &config["ui"]);
    update_ui(&config["settings"], values, cpus, top_mems, top_cpus, stash_fs);

    window.add(&vbox);
    window.show_all();
}

fn add_standard(item: &yaml_rust::Yaml, inner_box: &gtk::Box) -> (gtk::Label, Option<gtk::ProgressBar>) {
    // let deet = deets::do_func(item);

    let line_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
    line_box.get_style_context().add_class("row");

    let key = gtk::Label::new(None);
    key.get_style_context().add_class("key");
    key.set_text(&format!("{}", item["text"].as_str().unwrap()));

    let val = gtk::Label::new(None);
    val.set_justify(gtk::Justification::Right);
    val.set_halign(gtk::Align::End);
    val.get_style_context().add_class("val");

    line_box.add(&key);
    line_box.pack_start(&val, true, true, 0);

    let mut p = None;

    match item["widget"].as_str() {
        Some("bar") => {
            let progress = gtk::ProgressBar::new();
            progress.set_hexpand(true);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
            vbox.add(&line_box);
            vbox.add(&progress);
            inner_box.add(&vbox);
            p = Some(progress);
        },
        _ => {
            inner_box.add(&line_box);
        },
    }

    return (val, p);
}

fn add_cpus(inner_box: &gtk::Box, cpus: &mut Vec<Cpu>) {
    for i in 0..*deets::CPU_COUNT {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        vbox.get_style_context().add_class("row");

        let line_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);

        let key = gtk::Label::new(None);
        key.get_style_context().add_class("key");
        key.set_text(&format!("CPU{:02}", i));

        let val = gtk::Label::new(None);
        val.get_style_context().add_class("val");

        let pct = gtk::Label::new(None);
        pct.get_style_context().add_class("val");
        pct.get_style_context().add_class("pct");
        pct.set_justify(gtk::Justification::Right);
        pct.set_halign(gtk::Align::End);

        let progress = gtk::ProgressBar::new();
        progress.set_hexpand(true);
        progress.get_style_context().add_class("cpus-progress");

        line_box.add(&key);
        line_box.add(&val);
        line_box.pack_start(&pct, true, true, 0);

        vbox.add(&line_box);
        vbox.add(&progress);
        inner_box.add(&vbox);

        cpus.push(Cpu {
            mhz: val,
            progress: progress,
            pct_label: pct,
        });
    }
}

fn add_consumers(uniq_item: &str, container: &gtk::Box, mems: &mut Vec<TopRow>) {
    container.get_style_context().add_class("top-frame");
    container.set_orientation(gtk::Orientation::Horizontal);

    let columns = [
        gtk::Box::new(gtk::Orientation::Vertical, SPACING),
        gtk::Box::new(gtk::Orientation::Vertical, SPACING),
        gtk::Box::new(gtk::Orientation::Vertical, SPACING),
    ];

    fn add_to_column(i: usize, label: &gtk::Label, columns: &[gtk::Box; 3]) {
        match i {
            0 => {
                label.set_halign(gtk::Align::Start);
                columns[0].pack_start(label, true, true, 0)
            },
            1 => {
                columns[i].add(label);
                label.set_halign(gtk::Align::End)
            },
            2 => {
                columns[i].add(label);
                label.set_halign(gtk::Align::End)
            },
            _ => (),
        }
    }

    // for (i, name) in [ "NAME-------------", "------PID", &format!("-----{}", uniq_item) ].iter().enumerate() {
    for (i, name) in [ "NAME             ", "      PID", &format!("     {}", uniq_item) ].iter().enumerate() {
        let label = gtk::Label::new(None);
        label.set_text(&name);
        add_to_column(i, &label, &columns);
    }

    for _ in 0..5 {
        let mut tmp: Vec<gtk::Label> = Vec::new();

        for i in 0..3 {
            let label = gtk::Label::new(None);
            add_to_column(i, &label, &columns);
            tmp.push(label);
        }

        mems.push(TopRow {
            name: tmp[0].clone(),
            pid:  tmp[1].clone(),
            pct:  tmp[2].clone(),
        });
    }

    container.pack_start(&columns[0], true, true, 0);
    container.add(&columns[1]);
    container.add(&columns[2]);
}

fn init_ui(values: &mut HashMap<yaml_rust::Yaml, (gtk::Label, Option<gtk::ProgressBar>)>,
           cpus: &mut Vec<Cpu>,
           top_mems: &mut Vec<TopRow>,
           top_cpus: &mut Vec<TopRow>,
           stash_fs: &mut HashMap<String, (gtk::Label, gtk::ProgressBar)>,
           vbox: &gtk::Box,
           ui_config: &yaml_rust::Yaml) {

    for i in ui_config.as_vec().unwrap() {
        let label = Some(i["text"].as_str().unwrap());
        let frame = gtk::Frame::new(label);
        frame.get_style_context().add_class("frame");
        vbox.add(&frame);

        let inner_box = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        inner_box.get_style_context().add_class("innerbox");
        frame.add(&inner_box);

        if !i["type"].is_badvalue() {
            match i["type"].as_str().unwrap() {
                "cpus" => add_cpus(&inner_box, cpus),
                "mem_consumers" => add_consumers("MEM", &inner_box,  top_mems),
                "cpu_consumers" => add_consumers("CPU", &inner_box,  top_cpus),
                "filesystem" =>    add_filesystem(&inner_box, i["items"].as_vec().unwrap_or(&Vec::new()), stash_fs),
                "system" => {
                    for item in i["items"].as_vec().unwrap_or(&Vec::new()) {
                        let val = add_standard(item, &inner_box);
                        values.insert(item.clone(), val);
                    }
                }
                _ => (),
            }
        }
    }
}

fn add_filesystem(container: &gtk::Box, items: &Vec<Yaml>, stash: &mut HashMap<String, (gtk::Label, gtk::ProgressBar)>) {
    container.set_orientation(gtk::Orientation::Vertical);

    fn _add_item(container: &gtk::Box, item: &Yaml, stash: Option<&mut HashMap<String, (gtk::Label, gtk::ProgressBar)>>) {
        let columns = [
            gtk::Box::new(gtk::Orientation::Vertical, SPACING),
            gtk::Box::new(gtk::Orientation::Vertical, SPACING),
        ];

        let wrapper = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        let text = gtk::Label::new(None);
        text.set_text(item["text"].as_str().unwrap());
        columns[0].add(&text);

        let space = gtk::Label::new(None);
        space.set_halign(gtk::Align::End);
        columns[1].add(&space);

        wrapper.add(&columns[0]);
        wrapper.pack_start(&columns[1], true, true, 0);
        container.add(&wrapper);

        match stash {
            Some(s) => {
                let progress = gtk::ProgressBar::new();
                progress.set_hexpand(true);
                container.add(&progress);
                s.insert(String::from(item["mount_point"].as_str().unwrap()), (space, progress));
            },
            None => (),
        }
    }

    // _add_item(container, None, None);
    for item in items {
        _add_item(container, item, Some(stash));
    }
}

fn update_ui(config: &Yaml,
             values: HashMap<yaml_rust::Yaml, (gtk::Label, Option<gtk::ProgressBar>)>,
             cpus: Vec<Cpu>,
             top_mems: Vec<TopRow>,
             top_cpus: Vec<TopRow>,
             stash_fs: HashMap<String, (gtk::Label, gtk::ProgressBar)>) {

    fn do_top(ps_info: &Vec<deets::PsInfo>, top_ui_items: &Vec<TopRow>, member: &str) {
        for (i, lbl) in top_ui_items.iter().enumerate() {
            match member {
                "mem" => lbl.pct.set_text(&format!("{:.1}%", ps_info[i].mem)),
                "cpu" => lbl.pct.set_text(&format!("{:.1}%", ps_info[i].cpu)),
                _ => (),
            };

            lbl.pid.set_text(&format!("{}", ps_info[i].pid));

            let comm = &ps_info[i].comm;
            if comm.len() > 20 {
                lbl.name.set_text(&comm[0..20]);
            } else {
                lbl.name.set_text(comm);
            }
        }
    }

    let timeout = config["timeout"].as_i64().unwrap_or(1);
    let mod_top = config["skip_top"].as_i64().unwrap_or(2) as u64;
    let mod_fs  = config["skip_fs"].as_i64().unwrap_or(5)  as u64;

    let update = move || {
        let mut frame_counter = FRAME_COUNT.lock().unwrap();
        let mut frame_cache = deets::get_frame_cache(*frame_counter % 2 == 0);
        let cpu_mhz_vec = deets::get_cpu_mhz();
        let cpu_mhz_vec_len = cpu_mhz_vec.len();

        if &top_cpus.len() > &0 || &top_mems.len() > &0 {
            if *frame_counter % mod_top == 0 {
                frame_cache.ps_info.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap());
                do_top(&frame_cache.ps_info, &top_cpus, "cpu");

                frame_cache.ps_info.sort_by(|a, b| b.mem.partial_cmp(&a.mem).unwrap());
                do_top(&frame_cache.ps_info, &top_mems, "mem");
            }
        }

        if stash_fs.len() != 0 && (*frame_counter % mod_fs == 0) {
            #[cfg(feature = "timings")]
            use std::time::{Instant};
            #[cfg(feature = "timings")]
            let now = Instant::now();

            let fs_usage = deets::get_fs(stash_fs.keys().map(|s| s.as_str()).collect::<Vec<&str>>());

            #[cfg(feature = "timings")]
            println!("fs_usage:      millis: {}\tnanos: {}", now.elapsed().as_millis(), now.elapsed().as_nanos());

            for (k, v) in fs_usage.iter() {
                let stash = stash_fs.get(k).unwrap();
                stash.0.set_text(&format!("{} / {} {}", v.used_str, v.total_str, v.use_pct));
                stash.1.set_fraction(v.used / v.total);
            }
        }

        for (i, cpu) in cpus.iter().enumerate() {
            let usage = deets::get_cpu_usage(i as i32);

            if cpu_mhz_vec_len != 0 {
                cpu.mhz.set_text(&format!("{:04.0} MHz", cpu_mhz_vec[i]));
            }

            cpu.progress.set_fraction(usage / 100.0);
            cpu.pct_label.set_text(&format!("{:.0}%", usage));
        }

        for (item, val) in values.iter() {
            let func: &str = item["func"].as_str().unwrap();
            let deet = deets::do_func(item, &frame_cache);
            val.0.set_text(&deet.as_str());

            match &val.1 {
                Some(bar) => {
                    match func {
                        "cpu_usage" => bar.set_fraction(deets::get_cpu_usage(-1) / 100.0),
                        "ram_usage" => bar.set_fraction((frame_cache.mem_total - frame_cache.mem_free) / frame_cache.mem_total),
                        _ => (),
                    };
                },
                _ => (),
            }
        }

        *frame_counter += 1;
        return glib::Continue(true);
    };

    // update now!!
    update();

    #[cfg(feature = "benchmark")]
    {
        use std::time::{Instant};
        let bench_update = move || {
            let now = Instant::now();
            for _ in 0..1024 {
                update();
            }
            println!("millis: {}\tnanos: {}", now.elapsed().as_millis(), now.elapsed().as_nanos());
            return glib::Continue(true);
        };
        glib::timeout_add_seconds_local(timeout as u32, bench_update);
    }

    #[cfg(not(feature = "benchmark"))]
    glib::timeout_add_seconds_local(timeout as u32, update);
}

fn get_file() -> String {
    let input = args().collect::<Vec<String>>();
    let mut config_path = &String::from("./config/default.yml");

    if input.len() > 1 {
        config_path = &input[1];
        if !config_path.ends_with(".yml") && !config_path.ends_with(".yaml") {
            panic!("Need to provide a valid config path: {}", config_path);
        }
    }

    return match fs::read_to_string(config_path) {
        Ok(s)  => s,
        Err(_) => panic!("Unable to open/read {}", config_path),
    };
}

fn get_config(yaml_str: &str) -> Vec<Yaml> {
    let yaml = match YamlLoader::load_from_str(yaml_str) {
        Ok(y)  => y,
        Err(_) => panic!("Unable to parse config YAML"),
    };

    return yaml;
}

fn main() {
    let application = gtk::Application::new(Some("org.ahands.randy"), Default::default()).expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&Vec::new());
}
