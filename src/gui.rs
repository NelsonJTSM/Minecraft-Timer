//! # Clock Sample
//!
//! This sample demonstrates how to use gtk::timeout_add_seconds to run
//! a periodic task, implementing a clock in this example.

extern crate chrono;
extern crate gio;
extern crate gtk;

use gtk::prelude::*;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration,SystemTime};

pub fn build_ui(
    application: &gtk::Application,
    receiver: Arc<Mutex<mpsc::Receiver<minecraft_timer::Message>>>,
) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Minecraft timer");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(260, 40);

    let time = String::from("00:00:00");
    let label = gtk::Label::new(None);
    label.set_text(&time);

    window.add(&label);

    window.show_all();

    let mut _last_message: Arc<Mutex<Option<minecraft_timer::Message>>> = Arc::new(Mutex::new(None));

    // we are using a closure to capture the label (else we could also use a normal function)
    let tick = move || {
        let message = receiver
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_millis(100));

        match message {
            Ok(message) => Arc::clone(&_last_message).lock().unwrap().replace(message),
            _ => None,
        };

        // let x = Arc::clone(&last_message).lock().unwrap().as_ref();

        match Arc::clone(&_last_message).lock().unwrap().as_ref() {
            Some(time) => {
                let seconds_passed = SystemTime::now().duration_since(time.last_modified).unwrap().as_secs();
                let full_seconds = seconds_passed + time.seconds_played;
                label.set_text(&minecraft_timer::convert_seconds_to_hh_mm_ss(full_seconds));
            },
            _ => (),
        }

        glib::Continue(true)
    };

    // executes the closure once every second
    gtk::timeout_add_seconds(1, tick);
}
