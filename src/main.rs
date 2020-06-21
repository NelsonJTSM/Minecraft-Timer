extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

mod gui;

fn main() {
    let (sender, receiver) = mpsc::channel();

    let mut receiver = Arc::new(Mutex::new(receiver));

    // let mut sender = Arc::new(Mutex::new(sender));

    let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.clock"), Default::default())
            .expect("Initialization failed...");

    minecraft_timer::run(sender);

    application.connect_activate(move |app| {
        gui::build_ui(app, Arc::clone(&receiver));
    });

    application.run(&args().collect::<Vec<_>>());
}
