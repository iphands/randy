#[macro_use]
extern crate lazy_static;
extern crate gio;
extern crate gtk;
extern crate yaml_rust;

mod deets;

use yaml_rust::{YamlLoader, Yaml};

use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;
use std::fs;

use std::collections::HashMap;

const SPACING: i32 = 8;

fn get_css(conf: &Yaml) -> String {
    let css: String = String::from(include_str!("styles/app.css"));
    return css
        .replace("{ background-color }", conf["background_color"].as_str().unwrap())
        .replace("{ font-size }", conf["font-size"].as_str().unwrap());
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
    window.set_decorated(config["settings"]["decoration"].as_bool().unwrap());
    window.set_position(gtk::WindowPosition::Center);
    window.set_resizable(config["settings"]["resizable"].as_bool().unwrap());
    window.set_default_size(375, -1);

    // window.move_(3840 - 375 - 20 - 375, 20);
    // window.set_default_size(375, 2100);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
    vbox.get_style_context().add_class("container");

    let mut values = HashMap::new();
    let mut cpus = Vec::new();

    init_ui(&mut values, &mut cpus, &vbox, &config["ui"]);
    update_ui(config["settings"]["timeout"].as_i64().unwrap(), values, cpus);

    window.add(&vbox);
    window.show_all();
}

fn add_standard(item: &yaml_rust::Yaml, inner_box: &gtk::Box) -> gtk::Label {
    let deet = deets::do_func(item);

    let line_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);
    line_box.get_style_context().add_class("row");

    let key = gtk::Label::new(None);
    key.get_style_context().add_class("key");
    key.set_text(&format!("{}", item["text"].as_str().unwrap()));

    let val = gtk::Label::new(None);
    val.get_style_context().add_class("val");
    val.set_text(&deet.as_str());

    line_box.add(&key);
    line_box.add(&val);
    inner_box.add(&line_box);

    match item["widget"].as_str() {
        Some("bar") => {
            let progress = gtk::ProgressBar::new();
            progress.set_hexpand(true);
            line_box.add(&progress);
        },
        _ => (),
    }

    return val;
}

struct Cpu {
    mhz: gtk::Label,
    progress: gtk::ProgressBar,
}

fn add_cpus(item: &yaml_rust::Yaml, inner_box: &gtk::Box, cpus: &mut Vec<Cpu>) {
    for (i, _) in deets::get_cpu_mhz().iter().enumerate() {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, SPACING);
        vbox.get_style_context().add_class("row");

        let line_box = gtk::Box::new(gtk::Orientation::Horizontal, SPACING);

        let key = gtk::Label::new(None);
        key.get_style_context().add_class("key");
        key.set_text(&format!("CPU{:02}", i));

        let val = gtk::Label::new(None);
        val.get_style_context().add_class("val");
        val.set_text("---- MHz");

        let progress = gtk::ProgressBar::new();
        progress.set_hexpand(true);
        progress.get_style_context().add_class("cpus-progress");

        line_box.add(&key);
        line_box.add(&val);
        vbox.add(&line_box);
        vbox.add(&progress);
        inner_box.add(&vbox);

        cpus.push( Cpu {
            mhz: val,
            progress: progress,
        });
    }
}

fn init_ui(values: &mut HashMap<yaml_rust::Yaml, gtk::Label>,
           cpus: &mut Vec<Cpu>,
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

        for item in i["items"].as_vec().unwrap() {
            if item["type"].is_badvalue() {
                let val = add_standard(item, &inner_box);
                values.insert(item.clone(), val);
            } else {
                match item["type"].as_str().unwrap() {
                    "cpus" => add_cpus(item, &inner_box, cpus),
                    _ => (),
                }
            }
        }
    }
}

fn update_ui(timeout: i64, values: HashMap<yaml_rust::Yaml, gtk::Label>, cpus: Vec<Cpu>) {
    let update = move || {
        let cpu_mhz_vec = deets::get_cpu_mhz();

        for (i, cpu) in cpus.iter().enumerate() {
            let mhz = (cpu_mhz_vec[i] as u32).to_string();
            cpu.mhz.set_text(&format!("{} MHz", mhz));
            cpu.progress.set_fraction(deets::get_cpu_usage(i as i32) / 100.0)
        }

        for (item, val) in values.iter() {
            let func: &str = item["func"].as_str().unwrap();
            let deet = deets::do_func(item);
            val.set_text(&deet.as_str());

            // TODO this is shiiiitty
            // refactor so that the deets::do_func returns raw data

            if func == "ram_usage" {
                let parent: gtk::Box = val.get_parent().unwrap().downcast().unwrap();
                let tmp: &gtk::Widget = &parent.get_children()[2]; //.downcast::<gtk::ProgressBar>().unwrap();
                let progress = tmp.downcast_ref::<gtk::ProgressBar>().unwrap();

                let data: Vec<&str> = deet.split(" / ").collect(); // .map(String::from);
                let used = data[0].replace("GB", "").parse::<f64>().unwrap();
                let total = data[1].replace("GB", "").parse::<f64>().unwrap();
                progress.set_fraction(used / total);
            }

            if func == "cpu_usage" {
                let parent: gtk::Box = val.get_parent().unwrap().downcast().unwrap();
                let tmp: &gtk::Widget = &parent.get_children()[2]; //.downcast::<gtk::ProgressBar>().unwrap();
                let progress = tmp.downcast_ref::<gtk::ProgressBar>().unwrap();
                progress.set_fraction(deets::get_cpu_usage(-1) / 100.0);
            }
        }

        return glib::Continue(true);
    };

    glib::timeout_add_seconds_local(timeout as u32, update);
}

fn get_file() -> String {
    let input = args().collect::<Vec<String>>();
    let mut config_path = &String::from("./config/config.yml");

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
