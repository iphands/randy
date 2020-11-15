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

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    let s: &str = &get_file();
    let config = &get_config(s)[0];

    //Add custom CSS
    const CSS: &str = include_str!("styles/app.css");
    let screen = window.get_screen().unwrap();
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS.as_bytes()).expect("Failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(&screen, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    window.set_title("Randy");
    window.set_border_width(config["settings"]["border"].as_i64().unwrap() as u32);
    window.set_decorated(config["settings"]["decoration"].as_bool().unwrap());
    window.set_position(gtk::WindowPosition::Center);
    window.set_resizable(config["settings"]["resizable"].as_bool().unwrap());
    window.set_default_size(375, -1);

    // window.move_(3840 - 375 - 20 - 375, 20);
    // window.set_default_size(375, 2100);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let mut values = HashMap::new();

    init_ui(&mut values, &vbox, &config["ui"]);
    update_ui(config["settings"]["timeout"].as_i64().unwrap(), values);

    window.add(&vbox);
    window.show_all();
}

fn init_ui(values: &mut HashMap<String, gtk::Label>, vbox: &gtk::Box, ui_config: &yaml_rust::Yaml) {
    for i in ui_config.as_vec().unwrap() {
        let label = Some(i["text"].as_str().unwrap());
        let frame = gtk::Frame::new(label);
        frame.get_style_context().add_class("frame");
        vbox.add(&frame);

        let inner_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        inner_box.get_style_context().add_class("innerbox");
        frame.add(&inner_box);

        for item in i["items"].as_vec().unwrap() {
            let func = item["func"].as_str().unwrap();
            let deet = deets::do_func(item["func"].as_str().unwrap());
            // let text = item["text"].as_str().unwrap().replace("{}", deet.as_str());

            let line_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);

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

            values.insert(String::from(func), val);
        }
    }
}

fn update_ui(timeout: i64, values: HashMap<String, gtk::Label>) {
    let foo = move || {
        for (func, val) in values.iter() {
            let deet = deets::do_func(func);
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
                let data = deet.replace("%", "").parse::<f64>().unwrap();
                progress.set_fraction(data / 100.0);
            }
        }

        return glib::Continue(true);
    };

    glib::timeout_add_seconds_local(timeout as u32, foo);
}

fn get_file() -> String {
    let input = args().collect::<Vec<String>>();
    let mut config_path = &String::from("./config.yml");

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
        Err(_) => panic!("Unable to parse YAML from ./config.yml"),
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
