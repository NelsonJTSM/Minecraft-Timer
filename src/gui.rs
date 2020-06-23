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

    let last_message: Arc<Mutex<Option<minecraft_timer::Message>>> = Arc::new(Mutex::new(None));

    let last_message_clone1 = Arc::clone(&last_message);

    let update_world = move || {
        let message = receiver
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_millis(100));

        match message {
            Ok(message) => last_message_clone1.lock().unwrap().replace(message),
            _ => None,
        };

        glib::Continue(true)
    };

    let last_message_clone2 = Arc::clone(&last_message);

    // Gets called every Minecraft tick (every 1/20th of a second).
    // Used to update visual of the timer.
    let tick = move || {
        let label_text = match last_message_clone2.lock().unwrap().as_ref() {
            Some(message) => {
                /*
                if message.world.is_none() && message.player.is_none() {
                    String::from("00:00:00")
                }
                */
                
                let ticks_played = match message.player.as_ref() {
                    Some(player) => {
                        player.ticks_played
                    },
                    None => 0
                };

                let last_modified = match message.world.as_ref() {
                    Some(world) => {
                        world.last_modified
                    },
                    None => SystemTime::now()
                };

                minecraft_timer::get_time_difference(last_modified, ticks_played)
            },
            _ => String::from("00:00:00"),
        };

        label.set_text(&label_text);

        glib::Continue(true)
    };

    // executes the closure once every second
    gtk::timeout_add_seconds(1, update_world);
    gtk::timeout_add(50, tick);
}
