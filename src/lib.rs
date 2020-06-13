extern crate dirs;
extern crate notify;

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use notify::{watcher, DebouncedEvent, INotifyWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::time::Duration;

pub fn run() {
    let minecraft_folder = get_minecraft_folder_path();
    let saves_folder = minecraft_folder.join("saves");

    // Watcher variables
    let (stat_sender, stat_receiver) = channel();
    let mut stat_watcher = watcher(stat_sender, Duration::from_secs(1)).unwrap();

    let (new_world_sender, new_world_receiver) = channel();
    let mut new_world_watcher = watcher(new_world_sender, Duration::from_secs(1)).unwrap();

    let (stat_folder_sender, stat_folder_receiver) = channel();
    let mut stat_folder_watcher = watcher(stat_folder_sender, Duration::from_secs(1)).unwrap();

    let (new_player_sender, new_player_receiver) = channel();
    let mut new_player_watcher = watcher(new_player_sender, Duration::from_secs(1)).unwrap();

    new_world_watcher
        .watch(saves_folder, RecursiveMode::NonRecursive)
        .unwrap();

    /*
    stat_folder_watcher
        .watch(
            PathBuf::from("/home/town/.minecraft/saves/New World (14)/stats"),
            RecursiveMode::NonRecursive,
        )
        .unwrap();
    */

    loop {
        println!("bruh1");
        loop_new_world_watcher(&new_world_receiver, &mut stat_folder_watcher);
        println!("bruh2");
        loop_stat_folder_watcher(&stat_folder_receiver, &mut new_player_watcher);
        println!("bruh3");
        loop_new_player_watcher(&new_player_receiver, &mut stat_watcher);
        loop_stat_watcher(&stat_receiver);
        println!("he");
    }
}

fn loop_new_player_watcher(receiver: &Receiver<DebouncedEvent>, watcher: &mut INotifyWatcher) {
    match receiver.recv() {
        Ok(event) => match event {
            DebouncedEvent::Create(new_player_stat) => {
                println!("new_player_stat: {:?}", new_player_stat);
                watcher
                    .watch(new_player_stat, RecursiveMode::NonRecursive)
                    .unwrap();
            }
            e => println!("new_player_watcher: {:?}", e),
        },
        Err(e) => println!("watch error: {:?}", e),
    }
}

fn loop_new_world_watcher(receiver: &Receiver<DebouncedEvent>, watcher: &mut INotifyWatcher) {
    match receiver.recv() {
        Ok(event) => match event {
            DebouncedEvent::Create(new_world_directory) => {
                println!("new world made! {:?}", new_world_directory);
                println!(
                    "{:?}",
                    watcher
                        .watch(new_world_directory, RecursiveMode::NonRecursive)
                        .unwrap()
                );
            }
            _ => (),
        },
        Err(e) => println!("watch error: {:?}", e),
    }
}

fn loop_stat_folder_watcher(receiver: &Receiver<DebouncedEvent>, watcher: &mut INotifyWatcher) {
    match receiver.recv() {
        Ok(event) => {
            println!("event: {:?}", event);

            match event {
                DebouncedEvent::Create(new_file) => {
                    println!("new stat folder? {:?}", new_file);
                    if new_file.file_name().unwrap() == "stats" {
                        println!("new stat folder {:?}", new_file);
                        watcher
                            .watch(new_file, RecursiveMode::NonRecursive)
                            .unwrap();
                    }
                }
                e => println!(".....: {:?}", e),
            }
        }
        Err(e) => println!("watch error: {:?}", e),
    }
}

fn loop_stat_watcher(receiver: &Receiver<DebouncedEvent>) {
    match receiver.recv() {
        Ok(event) => match event {
            DebouncedEvent::Write(player_stat) => {
                println!("loop_stat_watcher {:?}", player_stat);

                let player_stat_json =
                    fs::read_to_string(player_stat).expect("Error trying to read file");

                display_player_stat(player_stat_json);
            }
            e => println!("loop_stat_watcher {:?}", e),
        },
        Err(e) => println!("watch error: {:?}", e),
    }
}

fn display_player_stat(player_json: String) {
    let p = Player::new(player_json);
    println!("Time = {}", p.seconds_played());
}

fn get_minecraft_folder_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not get home directory")
        .join(".minecraft")
}

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
    pub fn new(data: String) -> Player {
        serde_json::from_str(&remove_minecraft_prefix(data))
            .expect("Error trying to parse Player's json")
    }

    pub fn seconds_played(&self) -> f64 {
        self.stats.custom.play_one_minute as f64 / 20.0
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
        // JSON taken from the stats folder in a random world.
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

        assert_eq!(
            p.stats.custom.play_one_minute, 119,
            "Player's play_one_minute is being parsed correctly"
        );
        assert_eq!(
            p.seconds_played(),
            5.95,
            "Player's seconds played is not being calculated correctly"
        );
    }
}
