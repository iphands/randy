extern crate gio;
extern crate gtk;

mod deets;

use yaml_rust::{YamlLoader, Yaml};

use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;
use std::fs;

fn build_ui(application: &gtk::Application) {
    let s: &str = &get_file();
    let yaml = &get_config(s)[0];

    let window = gtk::ApplicationWindow::new(application);

    window.set_title("First GTK+ Program");
    window.set_border_width(0);
    window.set_decorated(false);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(375, 2100);
    window.set_resizable(false);
    window.move_(3840 - 375 - 20 - 375, 20);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);


    for i in yaml.as_vec().unwrap() {
        let label = gtk::Label::new(None);
        label.set_text(&format!("> {}", i["text"].as_str().unwrap()));
        vbox.add(&label);

        for item in i["items"].as_vec().unwrap() {
            let deet = deets::do_func(item["func"].as_str().unwrap());
            let text = item["text"].as_str().unwrap().replace("{}", deet.as_str());
            let label = gtk::Label::new(None);
            label.set_text(&format!("{}", text));
            vbox.add(&label);
        }
    }

    window.add(&vbox);
    window.show_all();
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
