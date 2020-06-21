//! # Clock Sample
//!
//! This sample demonstrates how to use gtk::timeout_add_seconds to run
//! a periodic task, implementing a clock in this example.

extern crate chrono;
extern crate gio;
extern crate gtk;

use chrono::Local;
use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn build_ui(
    application: &gtk::Application,
    receiver: Arc<Mutex<mpsc::Receiver<minecraft_timer::Message>>>,
) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("First GTK+ Clock");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(260, 40);

    let time = String::from("12");
    let label = gtk::Label::new(None);
    label.set_text(&time);

    window.add(&label);

    window.show_all();

    /*
    let time_change_thread = thread::spawn(move || loop {
        let message = receiver.lock().unwrap().recv().unwrap();
        // thread::sleep(Duration::from_millis(50));
        label.set_text(&message.minecraft_time);
    });
    */

    // time_change_thread.join().unwrap();

    // we are using a closure to capture the label (else we could also use a normal function)
    let tick = move || {
        // let time = String::from("12");
        let message = receiver
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_millis(100));
        match message {
            Ok(message)=> label.set_text(&message.minecraft_time),
            _ => (),
        }

        //    let r = receiver.lock().unwrap();

        // we could return glib::Continue(false) to stop our clock after this tick
        println!("bruh");
        glib::Continue(true)
    };

    // executes the closure once every second
    gtk::timeout_add_seconds(1, tick);
}
