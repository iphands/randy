use gio::prelude::*;
use gtk::prelude::*;
use lazy_static::lazy_static;

use std::{
    fs,
    env,
    sync::Mutex,
    time::Instant,
    collections::HashMap, 
};

use yaml_rust::{
    Yaml,
    YamlLoader,
};

use randy::*; // To use functions defined in lib.rs

struct UiStash {
    batts: HashMap<String, Battery>,
    cpus: Vec<Cpu>,
    gpus: Vec<Gpu>,
    fs: HashMap<String, (gtk::Label, gtk::ProgressBar)>,
    net: HashMap<String, (gtk::Label, gtk::Label)>,
    system: HashMap<yaml_rust::Yaml, (gtk::Label, Option<gtk::ProgressBar>)>,
    top_mems: Vec<TopRow>,
    top_cpus: Vec<TopRow>,
}

const SPACING: i32 = 3;

struct Cpu {
    mhz: gtk::Label,
    progress: gtk::ProgressBar,
    pct_label: gtk::Label,
}

#[allow(dead_code)]
struct Gpu {
    model: gtk::Label,
    temp: gtk::Label,
    fan_speed: gtk::Label,
}

struct TopRow {
    name: gtk::Label,
    pid: gtk::Label,
    pct: gtk::Label,
}

struct Battery {
    lbl_pct:          gtk::Label,
    lbl_status:       gtk::Label,
    str_battery:      String,
    str_plugged:      String,
    str_pct_template: String,
}

struct NetDevCache {
    last_bytes: u64,
    last_instant: Instant,
}

lazy_static! {
    static ref FRAME_COUNT: Mutex<u64> = Mutex::new(0);
}

fn get_css(conf: &Yaml, composited: bool) -> String {
    let css: String = String::from(include_str!("styles/app.css"));
    let color_text = conf["color_text"].as_str().unwrap_or("#e1eeeb");
    let color_background = conf["color_background"].as_str().unwrap_or("rgba(0, 0, 0, 0.5)");
    let color_trough = match composited {
        true  => conf["color_trough"].as_str().unwrap_or("rgba(0, 0, 0, 0)"),
        false => conf["color_trough"].as_str().unwrap_or(color_background),
    };

    let font_size = conf["font_size"].as_str().unwrap_or("large");
    let base_opacity = format!("{:1.4}", conf["base_opacity"].as_f64().unwrap_or(1.0));

    css
        .replace("{ bar_height }",       conf["bar_height"].as_str().unwrap_or("10px"))
        .replace("{ base_opacity }",     &base_opacity)
        .replace("{ color }",            conf["color_text"].as_str().unwrap_or("#e1eeeb"))
        .replace("{ color_background }", color_background)
        .replace("{ color_borders }",    conf["color_borders"].as_str().unwrap_or(color_text))
        .replace("{ color_bar }",        conf["color_bar"].as_str().unwrap_or("#e1eeff"))
        .replace("{ color_bar_med }",    conf["color_bar_med"].as_str().unwrap_or("#ffeeaa"))
        .replace("{ color_bar_high }",   conf["color_bar_high"].as_str().unwrap_or("#ffaaaa"))
        .replace("{ color_label }",      conf["color_label"].as_str().unwrap_or("#87d7ff"))
        .replace("{ color_trough }",     color_trough)
        .replace("{ font_family }",      conf["font_family"].as_str().unwrap_or("monospace"))
        .replace("{ font_size_top }",    conf["font_size_top"].as_str().unwrap_or(font_size))
        .replace("{ font_size }",        font_size)
}

fn _is_interactive(config: &Yaml) -> bool {
    config["decoration"].as_bool().unwrap_or(false) || config["resizable"].as_bool().unwrap_or(false)
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    let s: &str = &get_file();
    let config = &get_config(s)[0];
    let screen = gtk::prelude::WidgetExt::screen(&window).expect("Failed get Gtk Screen!");

    let css: &str = &get_css(&config["settings"], screen.is_composited());
    let provider = gtk::CssProvider::new();
    provider.load_from_data(css.as_bytes()).expect("Failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(&screen, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    window.set_title("Randy");
    window.set_decorated(config["settings"]["decoration"].as_bool().unwrap_or(false));
    window.set_resizable(config["settings"]["resizable"].as_bool().unwrap_or(false));
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(375, -1);
    window.set_skip_taskbar_hint(config["settings"]["skip_taskbar"].as_bool().unwrap_or(true));
    window.set_keep_below(!_is_interactive(&config["settings"]));
    window.set_accept_focus(_is_interactive(&config["settings"]));
    // println!("Debug {:?}", _is_interactive(&config["settings"]));
    // println!("Debug {:?}", &config["settings"]);

    window.realize();

    let screen = gtk::prelude::WidgetExt::screen(&window).expect("Failed get Gtk Screen!");
    let visual = screen.rgba_visual().unwrap();
    window.set_visual(Some(&visual));

    if !config["settings"]["xpos"].is_badvalue() &&
        !config["settings"]["ypos"].is_badvalue() {
            window.move_(
                config["settings"]["xpos"].as_i64().unwrap() as i32,
                config["settings"]["ypos"].as_i64().unwrap() as i32,
            );
        }

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    vbox.style_context().add_class("container");

    let mut stash = UiStash {
        batts: HashMap::new(),
        system: HashMap::new(),
        cpus: Vec::new(),
        gpus: Vec::new(),
        net: HashMap::new(),
        top_mems: Vec::new(),
        top_cpus: Vec::new(),
        fs: HashMap::new(),
    };

    init_ui(&mut stash, &vbox, &config["ui"]);
    window.add(&vbox);
    window.show_all();
    update_ui(&config["settings"], stash);
}

fn add_standard(item: &yaml_rust::Yaml, inner_box: &gtk::Box) -> (gtk::Label, Option<gtk::ProgressBar>) {
    // let deet = deets::do_func(item);

    let line_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
    line_box.style_context().add_class("row");

    let key = gtk::Label::new(None);
    key.style_context().add_class("key");
    key.set_text(item["text"].as_str().unwrap());

    let val = gtk::Label::new(None);
    val.set_justify(gtk::Justification::Right);
    val.set_halign(gtk::Align::End);
    val.style_context().add_class("val");

    line_box.add(&key);
    line_box.pack_start(&val, true, true, 0);

    let mut prog = None;

    match item["widget"].as_str() {
        Some("bar") => {
            let progress = gtk::ProgressBar::new();
            progress.set_hexpand(true);
            progress.set_sensitive(false);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
            vbox.add(&line_box);
            vbox.add(&progress);
            inner_box.add(&vbox);
            prog = Some(progress);
        },
        _ => {
            inner_box.add(&line_box);
        },
    }

    (val, prog)
}

fn _add_cpus_regular(inner_box: &gtk::Box, cpus: &mut Vec<Cpu>) {
    for i in 0..*deets::CPU_COUNT {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        vbox.style_context().add_class("row");

        let line_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);

        let key = gtk::Label::new(None);
        key.style_context().add_class("key");
        key.set_text(&format!("CPU{:02}", i));

        let val = gtk::Label::new(None);
        val.style_context().add_class("val");

        let pct = gtk::Label::new(None);
        pct.style_context().add_class("val");
        pct.style_context().add_class("pct");
        pct.set_justify(gtk::Justification::Right);
        pct.set_halign(gtk::Align::End);

        let progress = gtk::ProgressBar::new();
        progress.set_hexpand(true);
        progress.style_context().add_class("cpus-progress");
        progress.set_sensitive(false);

        line_box.add(&key);
        line_box.add(&val);
        line_box.pack_start(&pct, true, true, 0);

        vbox.add(&line_box);
        vbox.add(&progress);
        inner_box.add(&vbox);

        cpus.push(Cpu {
            mhz: val,
            progress,
            pct_label: pct,
        });
    }
}

fn _add_cpus_split(inner_box: &gtk::Box, cpus: &mut Vec<Cpu>) {
    let left_box  = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    left_box.style_context().add_class("innerbox");

    let right_box = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    right_box.style_context().add_class("innerbox");

    for i in 0..*deets::CPU_COUNT / 2 {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        vbox.style_context().add_class("row");

        let line_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);

        let key = gtk::Label::new(None);
        key.style_context().add_class("key");
        key.set_text(&format!("CPU{:02}", i));

        let val = gtk::Label::new(None);
        val.style_context().add_class("val");

        let pct = gtk::Label::new(None);
        pct.style_context().add_class("val");
        pct.style_context().add_class("pct");
        pct.set_justify(gtk::Justification::Right);
        pct.set_halign(gtk::Align::End);

        let progress = gtk::ProgressBar::new();
        progress.set_hexpand(true);
        progress.style_context().add_class("cpus-progress");
        progress.set_sensitive(false);

        line_box.add(&key);
        line_box.add(&val);
        line_box.pack_start(&pct, true, true, 0);

        vbox.add(&line_box);
        vbox.add(&progress);
        left_box.add(&vbox);

        cpus.push(Cpu {
            mhz: val,
            progress,
            pct_label: pct,
        });
    }

    for i in *deets::CPU_COUNT / 2..*deets::CPU_COUNT {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        vbox.style_context().add_class("row");

        let line_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);

        let key = gtk::Label::new(None);
        key.style_context().add_class("key");
        key.set_text(&format!("CPU{:02}", i));

        let val = gtk::Label::new(None);
        val.style_context().add_class("val");

        let pct = gtk::Label::new(None);
        pct.style_context().add_class("val");
        pct.style_context().add_class("pct");
        pct.set_justify(gtk::Justification::Right);
        pct.set_halign(gtk::Align::End);

        let progress = gtk::ProgressBar::new();
        progress.set_hexpand(true);
        progress.style_context().add_class("cpus-progress");
        progress.set_sensitive(false);

        line_box.add(&key);
        line_box.add(&val);
        line_box.pack_start(&pct, true, true, 0);

        vbox.add(&line_box);
        vbox.add(&progress);
        right_box.add(&vbox);

        cpus.push(Cpu {
            mhz: val,
            progress,
            pct_label: pct,
        });
    }

    inner_box.set_orientation(gtk::Orientation::Horizontal);
    inner_box.add(&left_box);
    inner_box.add(&right_box);
}

fn add_cpus(inner_box: &gtk::Box, cpus: &mut Vec<Cpu>, split: bool) {
    if split {
	    _add_cpus_split(inner_box, cpus);
    } else {
        _add_cpus_regular(inner_box, cpus);
    }
}

fn add_gpus(inner_box: &gtk::Box, gpus: &mut Vec<Gpu>) {

    let gpu_count = 1;

    for _i in 0..gpu_count {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        vbox.style_context().add_class("row");

        let line_box1 = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        let line_box2 = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        let line_box3 = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);

        let key1 = gtk::Label::new(None);
        key1.style_context().add_class("key");
        key1.set_text("Model:");
        key1.set_halign(gtk::Align::Start);
        key1.set_hexpand(true);

        let model = gtk::Label::new(None);
        model.style_context().add_class("val");
        model.set_justify(gtk::Justification::Right);
        model.set_halign(gtk::Align::End);

        let key2 = gtk::Label::new(None);
        key2.style_context().add_class("key");
        key2.set_text("Temperature:");
        key2.set_halign(gtk::Align::Start);
        key2.set_hexpand(true);

        let temp = gtk::Label::new(None);
        temp.style_context().add_class("val");
        temp.set_justify(gtk::Justification::Right);
        temp.set_halign(gtk::Align::End);

        let key3 = gtk::Label::new(None);
        key3.style_context().add_class("key");
        key3.set_text("Fan Speed:");
        key3.set_halign(gtk::Align::Start);
        key3.set_hexpand(true);

        let fan_speed = gtk::Label::new(None);
        fan_speed.style_context().add_class("val");
        fan_speed.set_justify(gtk::Justification::Right);
        fan_speed.set_halign(gtk::Align::End);

        line_box1.add(&key1);
        line_box1.add(&model);

        line_box2.add(&key2);
        line_box2.add(&temp);

        line_box3.add(&key3);
        line_box3.add(&fan_speed);

        vbox.add(&line_box1);
        vbox.add(&line_box2);
        vbox.add(&line_box3);

        inner_box.add(&vbox);

        gpus.push(Gpu {
            model,
            temp,
            fan_speed,
        });
    }
}

fn add_consumers(uniq_item: &str, limit: i64, container: &gtk::Box, mems: &mut Vec<TopRow>) {
    container.style_context().add_class("top-frame");
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

    for (i, name) in ["NAME", "PID", uniq_item].iter().enumerate() {
        let label = gtk::Label::new(None);
        label.set_text(name);
        add_to_column(i, &label, &columns);
    }

    for _ in 0..limit {
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

fn init_ui(stash: &mut UiStash,
           vbox: &gtk::Box,
           ui_config: &yaml_rust::Yaml) {

    for yaml in ui_config.as_vec().unwrap() {
        let label = Some(yaml["text"].as_str().unwrap());
        let frame = gtk::Frame::new(label);
        frame.style_context().add_class("frame");
        vbox.add(&frame);

        let inner_box = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        inner_box.style_context().add_class("innerbox");
        frame.add(&inner_box);

        if !yaml["type"].is_badvalue() {
            let limit = yaml["limit"].as_i64().unwrap_or(5);
            match yaml["type"].as_str().unwrap() {
                "battery"       => add_batt(&inner_box, yaml["items"].as_vec().unwrap_or(&Vec::new()), &mut stash.batts),
                "cpus"          => add_cpus(&inner_box, &mut stash.cpus, yaml["split"].as_bool().unwrap_or(false)),
                "gpus"          => add_gpus(&inner_box, &mut stash.gpus),
                "mem_consumers" => add_consumers("MEM", limit, &inner_box, &mut stash.top_mems),
                "cpu_consumers" => add_consumers("CPU", limit, &inner_box, &mut stash.top_cpus),
                "filesystem"    => add_filesystem(&inner_box, yaml["items"].as_vec().unwrap_or(&Vec::new()), &mut stash.fs),
                "net"           => add_net(&inner_box, yaml["items"].as_vec().unwrap_or(&Vec::new()), &mut stash.net),
                "system" => {
                    for item in yaml["items"].as_vec().unwrap_or(&Vec::new()) {
                        let val = add_standard(item, &inner_box);
                        stash.system.insert(item.clone(), val);
                    }
                }
                _ => (),
            }
        }
    }
}

fn add_batt(container: &gtk::Box, items: &[Yaml], stash: &mut HashMap<String, Battery>) {
    container.set_orientation(gtk::Orientation::Horizontal);
    container.style_context().add_class("batt");

    let key_col = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    let val_col = gtk::Box::new(gtk::Orientation::Vertical, SPACING);

    items.iter().for_each(|item| {
        let str_battery = item["battery_text"].as_str().unwrap();
        let str_plugged = item["pluggged_text"].as_str().unwrap();
        let str_pct_template = item["percent_template"].as_str().unwrap();

        let key = gtk::Label::new(None);
        key.style_context().add_class("key");
        key.set_text(&format!("{}:", item["name"].as_str().unwrap()));
        key.set_halign(gtk::Align::Start);
        key.set_hexpand(true);
        key_col.add(&key);

        let val_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        val_box.set_halign(gtk::Align::Start);

        let status_lbl = gtk::Label::new(None);
        status_lbl.style_context().add_class("val");
        status_lbl.style_context().add_class("emoji");
        status_lbl.set_halign(gtk::Align::Start);
        status_lbl.set_text(str_battery);

        let pct_lbl = gtk::Label::new(None);
        pct_lbl.style_context().add_class("val");
        pct_lbl.set_halign(gtk::Align::Start);
        pct_lbl.set_text(&str_pct_template.replace("{}", "000"));

        val_box.add(&status_lbl);
        val_box.add(&pct_lbl);
        val_col.add(&val_box);

        stash.insert(String::from(item["path"].as_str().unwrap()), Battery {
            lbl_pct:          pct_lbl,
            lbl_status:       status_lbl,
            str_battery:      String::from(str_battery),
            str_plugged:      String::from(str_plugged),
            str_pct_template: String::from(str_pct_template),
        });
    });

    container.add(&key_col);
    container.add(&val_col);
}

fn add_net(container: &gtk::Box, items: &[Yaml], stash: &mut HashMap<String, (gtk::Label, gtk::Label)>) {
    container.set_orientation(gtk::Orientation::Horizontal);
    container.style_context().add_class("net");

    let key_col  = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    let up_col   = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    let down_col = gtk::Box::new(gtk::Orientation::Vertical, SPACING);

    items.iter().for_each(|item| {
        let key = gtk::Label::new(None);
        key.style_context().add_class("key");
        key.set_text(&format!("{}:", item["name"].as_str().unwrap()));
        key.set_halign(gtk::Align::Start);
        key.set_hexpand(true);
        key_col.add(&key);

        let up_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        let up_lbl = gtk::Label::new(None);
        up_box.set_halign(gtk::Align::Start);
        up_lbl.set_halign(gtk::Align::Start);
        up_lbl.set_text("Up");
        up_box.add(&up_lbl);

        let up_val = gtk::Label::new(None);
        up_val.style_context().add_class("val");
        up_val.set_hexpand(true);
        up_val.set_halign(gtk::Align::End);
        up_val.set_text("0000.00 KB");
        up_box.add(&up_val);
        up_box.set_halign(gtk::Align::Fill);
        up_col.add(&up_box);

        let down_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        let down_lbl = gtk::Label::new(None);
        down_box.set_halign(gtk::Align::Start);
        down_lbl.set_halign(gtk::Align::Start);
        down_lbl.set_text("Down");
        down_lbl.set_width_chars(12);
        down_box.add(&down_lbl);

        let down_val = gtk::Label::new(None);
        down_val.style_context().add_class("val");
        down_val.set_hexpand(true);
        down_val.set_halign(gtk::Align::End);
        down_val.set_text("0000.00 KB");
        down_val.set_width_chars(12);
        down_box.add(&down_val);
        down_box.set_halign(gtk::Align::Fill);
        down_col.add(&down_box);

        stash.insert(String::from(item["interface"].as_str().unwrap()), (up_val, down_val));
    });

    container.add(&key_col);
    container.add(&up_col);
    container.add(&down_col);
}

fn add_filesystem(container: &gtk::Box, items: &[Yaml], stash: &mut HashMap<String, (gtk::Label, gtk::ProgressBar)>) {
    container.set_orientation(gtk::Orientation::Vertical);

    fn _add_item(container: &gtk::Box, item: &Yaml, stash: Option<&mut HashMap<String, (gtk::Label, gtk::ProgressBar)>>) {
        let columns = [
            gtk::Box::new(gtk::Orientation::Vertical, SPACING),
            gtk::Box::new(gtk::Orientation::Vertical, SPACING),
        ];

        let wrapper = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
        let text = gtk::Label::new(None);
        text.style_context().add_class("key");
        text.set_text(item["text"].as_str().unwrap());
        columns[0].add(&text);

        let space = gtk::Label::new(None);
        space.set_halign(gtk::Align::End);
        space.style_context().add_class("val");
        columns[1].add(&space);

        wrapper.add(&columns[0]);
        wrapper.pack_start(&columns[1], true, true, 0);
        container.add(&wrapper);

        if let Some(s) = stash {
            let progress = gtk::ProgressBar::new();
            progress.set_hexpand(true);
            progress.set_sensitive(false);
            
            container.add(&progress);
            s.insert(String::from(item["mount_point"].as_str().unwrap()), (space, progress));
        }
    }

    // _add_item(container, None, None);
    for item in items {
        _add_item(container, item, Some(stash));
    }
}

fn _update_bar(bar: &gtk::ProgressBar, fraction: f64) {
    if fraction > 0.80 {
        bar.style_context().remove_class("med");
        bar.style_context().add_class("high");
    } else if fraction > 0.50 {
        bar.style_context().add_class("med");
        bar.style_context().remove_class("high");
    } else {
        bar.style_context().remove_class("med");
        bar.style_context().remove_class("high");
    }

    bar.set_fraction(fraction);
}

fn update_ui(config: &Yaml, stash: UiStash) {

    fn do_top(ps_info: &[deets::PsInfo], top_ui_items: &[TopRow], member: &str) {
        for (i, lbl) in top_ui_items.iter().enumerate() {
            match member {
                "mem" => lbl.pct.set_text(&format!("{:.1}%", ps_info[i].mem)),
                "cpu" => lbl.pct.set_text(&format!("{:.1}%", ps_info[i].cpu)),
                _ => (),
            };

            lbl.pid.set_text(&ps_info[i].pid.to_string());

            let comm = &ps_info[i].comm;
            if comm.len() > 20 {
                lbl.name.set_text(&comm[0..20]);
            } else {
                lbl.name.set_text(comm);
            }
        }
    }

    let timeout = config["timeout"].as_i64().unwrap_or(1);
    let mod_top = config["mod_top"].as_i64().unwrap_or(2) as u64;
    let mod_fs  = config["mod_fs"].as_i64().unwrap_or(2)  as u64;
    let mod_bat = config["mod_bat"].as_i64().unwrap_or(2) as u64;

    let get_fs = deets::get_fs;
    let get_mhz = deets::get_cpu_mhz;
    let get_battery = deets::get_battery;
    let mut net_cache: HashMap<String, NetDevCache> = HashMap::new();

    fn _get_net_bps(cache: &mut HashMap<String, NetDevCache>, key: &str, curr_bytes: &u64) -> String {
        if !cache.contains_key(key) {
            cache.insert(String::from(key), NetDevCache {
                last_bytes: *curr_bytes,
                last_instant: Instant::now(),
            });
        }

        let cache_val = cache.get(key).unwrap();
        let mut lbl = "KB";
        let mut bytes = (curr_bytes - cache_val.last_bytes) as f64 / 1024.0;
        bytes = (bytes * 1000.0) / (cache_val.last_instant.elapsed().as_millis() as f64);

        if bytes > 990.0 {
            bytes /= 1024.0;
            lbl = "MB";
        }

        if bytes > 990.0 {
            bytes /= 1024.0;
            lbl = "GB";
        }

        cache.insert(String::from(key), NetDevCache {
            last_bytes: *curr_bytes,
            last_instant: Instant::now(),
        });

        format!("{:.2} {}", bytes, lbl)
    }

    let mut update = move || {
        let mut frame_counter = FRAME_COUNT.lock().unwrap();
        let should_top = match stash.top_cpus.len() + stash.top_mems.len() {
            0 => false,
            _ => *frame_counter % mod_top == 0,
        };

        let mut frame_cache = deets::get_frame_cache(*frame_counter, mod_top, should_top);
        let cpu_mhz_vec = timings!("cpu_mhz", get_mhz);
        let cpu_mhz_vec_len = cpu_mhz_vec.len();

        if should_top {
            frame_cache.ps_info.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap());
            do_top(&frame_cache.ps_info, &stash.top_cpus, "cpu");

            frame_cache.ps_info.sort_by(|a, b| b.mem.partial_cmp(&a.mem).unwrap());
            do_top(&frame_cache.ps_info, &stash.top_mems, "mem");
        }

        if !stash.batts.is_empty() && (*frame_counter % mod_bat == 0) {
            stash.batts.iter().for_each(|(path, battery)| {
                let (plugged, pct) = timings!("battery", get_battery, path);
                battery.lbl_status.set_text(match plugged { true => &battery.str_plugged, false => &battery.str_battery, });
                battery.lbl_pct.set_text(&battery.str_pct_template.replace("{}", &pct));
            });
        }

        if !stash.net.is_empty() {
            stash.net.iter().for_each(|(interface, (up_lbl, down_lbl))| {
                if frame_cache.net_dev.contains_key(interface) {
                    let (up, down) = frame_cache.net_dev.get(interface).unwrap();
                    up_lbl.set_text(&_get_net_bps(&mut net_cache, &format!("{} up", interface), up));
                    down_lbl.set_text(&_get_net_bps(&mut net_cache, &format!("{} down", interface), down));
                }
            });
        }

        if !stash.fs.is_empty() && (*frame_counter % mod_fs == 0) {
            let fs_usage = timings!("fs_usage", get_fs, stash.fs.keys().map(|s| s.as_str()).collect::<Vec<&str>>());
            fs_usage.iter().for_each(|(k, v)| {
                let stash = stash.fs.get(k).unwrap();
                stash.0.set_text(&format!("{} / {} {}", v.used_str, v.total_str, v.use_pct));
                _update_bar(&stash.1, v.used / v.total);
            });
        }

        stash.cpus.iter().enumerate().for_each(|(i, cpu)| {
            let usage = deets::get_cpu_usage(i as i32);

            if cpu_mhz_vec_len != 0 {
                cpu.mhz.set_text(&format!("{:04.0} MHz", cpu_mhz_vec[i]));
            }

            _update_bar(&cpu.progress, usage / 100.0);
            cpu.pct_label.set_text(&format!("{:3.0}%", usage));
            cpu.pct_label.set_width_chars(4);
        });

        #[cfg(feature = "nvidia")]
        stash.gpus.iter().enumerate().for_each(|(index, gpu)| {

            let info = deets::nvidia_gpu_info(index as u32);

            let model = &info["model"];
            let temp = &info["temp"];
            let fan_speed = &info["fan_speed"];

            gpu.model.set_text(&model);
            gpu.temp.set_text(&temp);
            gpu.fan_speed.set_text(&fan_speed);
        });

        stash.system.iter().for_each(|(item, val)| {
            let func: &str = item["func"].as_str().unwrap();
            let deet = deets::do_func(item, &frame_cache);
            val.0.set_text(deet.as_str());

            if let Some(bar) = &val.1 {
                match func {
                    "cpu_usage" => _update_bar(bar, deets::get_cpu_usage(-1) / 100.0),
                    "ram_usage" => _update_bar(bar, (frame_cache.mem_total - frame_cache.mem_free) / frame_cache.mem_total),
                    _ => (),
                };
            }
        });

        *frame_counter += 1;
        glib::Continue(true)
    };

    // update now!!
    update();

    #[cfg(feature = "runtime_bench")]
    {
        use std::time::Instant;
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

    #[cfg(not(feature = "runtime_bench"))]
    glib::timeout_add_seconds_local(timeout as u32, update);
}

fn try_get_file() -> Option<String> {

    if let Ok(home) = env::var("HOME") {
        let cfg = format!("{home}/.randy.yml");
        if std::path::Path::new(&cfg).exists() {
            return Some(cfg);
        }
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        let cfg = format!("{xdg}/.randy.yml");
        if std::path::Path::new(&cfg).exists() {
            return Some(cfg);
        }
    }

    if let Ok(home) = env::var("HOME") {
        let cfg = format!("{home}/.config/randy.yml");
        if std::path::Path::new(&cfg).exists() {
            return Some(cfg);
        }
    }

    let cfg = "/etc/randy.yml";

    if std::path::Path::new(cfg).exists() {
	    Some(cfg.to_string())
    } else {
        None
    }
}

fn get_file() -> String {
    let config_path = try_get_file()
	.expect(r#"Could not find a randy.yml config file.
Checked in this order:
- $HOME/.randy.yml
- $XDG_CONFIG_HOME/randy.yml
- $HOME/.config/randy.yml
- /etc/randy.yml

Please put a randy.yml config file in one of those places.
Exmples: https://github.com/iphands/randy/tree/main/config"#);
    println!("Using config file: {}", config_path);
    match fs::read_to_string(&config_path) {
        Ok(s)  => s,
        Err(e) => {
            eprintln!("Unable to open/read {config_path}");
            panic!("Error: {e}");
        },
    }
}

fn get_config(yaml_str: &str) -> Vec<Yaml> {
    match YamlLoader::load_from_str(yaml_str) {
        Ok(y)  => y,
        Err(e) => {
            eprintln!("Unable to parse config YAML");
            panic!("Error: {e}");
        },
    }
}

fn main() {
    let application = gtk::Application::new(
        Some("org.ahands.randy"),
        Default::default()
    );

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
