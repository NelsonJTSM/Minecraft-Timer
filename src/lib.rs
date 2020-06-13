extern crate dirs;
extern crate notify;

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::sync::mpsc::channel;
use std::time::Duration;

use std::path::PathBuf;

pub fn run() {
    let minecraft_folder = get_minecraft_folder_path();
    let saves_folder = minecraft_folder.join("saves");
    let mut saves: Vec<PathBuf> = Vec::new();

    // watch_saves_folder(&saves_folder, &mut saves);
    watch_stats(saves);
}

fn watch_stats(saves: Vec<PathBuf>) {}

fn get_minecraft_folder_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not get home directory")
        .join(".minecraft")
}

/*
fn watch_saves_folder(saves_folder: &PathBuf, saves: &mut Vec<PathBuf>) {
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
                match event {
                    DebouncedEvent::Create(new_world_path) => {
                        saves.push(new_world_path);
                    }
                    _ => (),
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
*/

// Minecraft stats parsing.

fn remove_minecraft_prefix(stats: String) -> String {
    stats.replace("minecraft:", "")
}

#[derive(Serialize, Deserialize, Debug)]
struct Custom {
    play_one_minute: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    custom: Custom,
}

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    stats: Stats,
}

impl Player {
    pub fn seconds_played(&self) -> f64 {
        self.stats.custom.play_one_minute as f64 / 20.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Result, Value};

    #[test]
    fn minecraft_folder_exists() {
        assert!(
            get_minecraft_folder_path().as_path().is_dir(),
            "Minecraft folder does not exists"
        );
    }

    #[test]
    fn removes_minecraft_prefix_from_stats() {
        let stats = String::from(
            r#"
        {
            "stats": {
                "minecraft:custom": {
                    "minecraft:jump": 2,
                    "minecraft:time_since_rest": 119,
                    "minecraft:play_one_minute": 119,
                    "minecraft:leave_game": 1,
                    "minecraft:time_since_death": 119,
                    "minecraft:walk_one_cm": 63
                }
            },
            "DataVersion": 2230
        }
        "#,
        );

        let expected = String::from(
            r#"
        {
            "stats": {
                "custom": {
                    "jump": 2,
                    "time_since_rest": 119,
                    "play_one_minute": 119,
                    "leave_game": 1,
                    "time_since_death": 119,
                    "walk_one_cm": 63
                }
            },
            "DataVersion": 2230
        }
        "#,
        );

        assert_eq!(remove_minecraft_prefix(stats), expected);
    }

    #[test]
    fn stats_typed_correctly() {
        // Some JSON input data as a &str. Maybe this comes from the user.
        let data = r#"
        {
            "stats": {
                "custom": {
                    "jump": 2,
                    "time_since_rest": 119,
                    "play_one_minute": 119,
                    "leave_game": 1,
                    "time_since_death": 119,
                    "walk_one_cm": 63
                }
            },
            "DataVersion": 2230
        }
        "#;
        

        let p: Player = serde_json::from_str(data).expect("Error trying to parse Player's json");

        assert_eq!(p.stats.custom.play_one_minute, 119);
        assert_eq!(p.seconds_played(), 5.95, "Player's seconds played is not being calculated correctly");
    }
}
