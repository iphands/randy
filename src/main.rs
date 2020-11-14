#[macro_use]
extern crate lazy_static;
extern crate gio;
extern crate gtk;

mod deets;

use yaml_rust::{YamlLoader, Yaml};

use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;
use std::fs;

use std::collections::HashMap;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Ronky");
    window.set_border_width(0);
    window.set_decorated(false);
    window.set_position(gtk::WindowPosition::Center);
    window.set_resizable(false);
    window.set_default_size(375, -1);

    // window.move_(3840 - 375 - 20 - 375, 20);
    // window.set_default_size(375, 2100);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let mut values = HashMap::new();

    init_ui(&mut values, &vbox);
    update_ui(values);

    window.add(&vbox);
    window.show_all();
}

fn init_ui(values: &mut HashMap<String, gtk::Label>, vbox: &gtk::Box) {
    let s: &str = &get_file();
    let yaml = &get_config(s)[0];

    for i in yaml.as_vec().unwrap() {
        let label = Some(i["text"].as_str().unwrap());
        let frame = gtk::Frame::new(label);
        vbox.add(&frame);
        // vbox.pack_start(&frame, false, false, 10);

        let inner_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        frame.add(&inner_box);

        for item in i["items"].as_vec().unwrap() {
            let func = item["func"].as_str().unwrap();
            let deet = deets::do_func(item["func"].as_str().unwrap());
            // let text = item["text"].as_str().unwrap().replace("{}", deet.as_str());

            let line_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);

            let key = gtk::Label::new(None);
            key.set_text(&format!("{}", item["text"].as_str().unwrap()));

            let val = gtk::Label::new(None);
            val.set_text(&deet.as_str());

            line_box.add(&key);
            line_box.add(&val);
            inner_box.add(&line_box);
            values.insert(String::from(func), val);
        }
    }
}

fn update_ui(values: HashMap<String, gtk::Label>) {
    let foo = move || {
        for (func, val) in values.iter() {
            let deet = deets::do_func(func);
            val.set_text(&deet.as_str());
        }
        return glib::Continue(true);
    };

    glib::timeout_add_seconds_local(1, foo);
}

fn get_file() -> String {
    return match fs::read_to_string("./config.yml") {
        Ok(s)  => s,
        Err(_) => panic!("fdsaf"),
    };
}

fn get_config(yaml_str: &str) -> Vec<Yaml> {
    let yaml = match YamlLoader::load_from_str(yaml_str) {
        Ok(y)  => y,
        Err(_) => panic!("fdsaf"),
    };

    return yaml;
}

fn main() {
    let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
        .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
