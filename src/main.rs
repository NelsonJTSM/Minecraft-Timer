extern crate gio;
extern crate gtk;

use gio::prelude::*;
use std::env::args;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

mod gui;

fn main() {
    let (sender, receiver) = mpsc::channel();

    let mut _receiver = Arc::new(Mutex::new(receiver));

    let application =
        gtk::Application::new(Some("net.nelsontorres.minecrafttimer"), Default::default())
            .expect("Initialization failed...");

    minecraft_timer::run(sender);

    application.connect_activate(move |app| {
        gui::build_ui(app, Arc::clone(&_receiver));
    });

    application.run(&args().collect::<Vec<_>>());
}
