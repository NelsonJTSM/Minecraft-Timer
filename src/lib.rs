extern crate dirs;
extern crate notify;

use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

use std::path::PathBuf;

pub fn run() {
    let minecraft_folder = get_minecraft_folder_path();
    let saves_folder = minecraft_folder.join("saves");

    watch_saves_folder(&saves_folder);
}

fn get_minecraft_folder_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not get home directory")
        .join(".minecraft")
}

fn watch_saves_folder(saves_folder: &PathBuf) {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch(saves_folder.as_os_str(), RecursiveMode::NonRecursive)
        .unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("{:?}", event);
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minecraft_folder_exists() {
        assert!(
            get_minecraft_folder_path().as_path().is_dir(),
            "Minecraft folder does not exists"
        );
    }
}
