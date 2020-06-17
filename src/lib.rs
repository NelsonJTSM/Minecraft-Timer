extern crate dirs;
extern crate notify;

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration};
use regex::Regex;

use serde::{Deserialize, Serialize};

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

pub fn run() {
    let minecraft_folder = get_minecraft_folder_path();
    let saves_folder = minecraft_folder.join("saves");

    let stat_thread = thread::spawn(|| {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();
        watcher
            .watch(saves_folder, RecursiveMode::Recursive)
            .unwrap();

        loop {
            match rx.recv() {
                Ok(event) => match event {
                    DebouncedEvent::Write(file) => {
                        if file.extension().unwrap() == "json"
                            && file.parent().unwrap().file_name().unwrap() == "stats"
                        {
                            display_player_stat(fs::read_to_string(file).unwrap());
                        }
                    }
                    _ => (),
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    stat_thread.join().unwrap();
}

fn convert_seconds_to_hh_mm_ss(time: u64) -> String {
    let mut time_left = time;
    let hours = time_left / 3600;
    time_left -= hours * 3600;
    let minutes = time_left / 60;
    time_left -= minutes * 60;
    let seconds = time_left;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn display_player_stat(player_json: String) {
    // let p = Player::new(player_json);
    // println!("{}", convert_seconds_to_hh_mm_ss(p.seconds_played() as u64));

    let seconds = get_seconds_played_from_stats(&player_json);
    println!("{}", convert_seconds_to_hh_mm_ss(seconds.unwrap()));
}

fn get_minecraft_folder_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not get home directory")
        .join(".minecraft")
}

// OLD:
// 
/*
fn remove_minecraft_prefix(stats: String) -> String {
    stats.replace("minecraft:", "")
}
*/

#[derive(Serialize, Deserialize, Debug)]
struct Custom {
    play_one_minute: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    custom: Custom,
}

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    stats: Stats,
}

/*
Old way to grab the time statistic from the stats file.

Does not work for version 1.7.2, since it uses a different format 
for the stats file.

impl Player {
    pub fn new(data: String) -> Player {
        serde_json::from_str(&remove_minecraft_prefix(data))
            .expect("Error trying to parse Player's json")
    }

    pub fn seconds_played(&self) -> f64 {
        self.stats.custom.play_one_minute as f64 / 20.0
    }
}
*/


// Uses a regex to grab the playedOneMinute statistic.
// Works for both Minecraft 1.7.2 and 1.15
fn get_seconds_played_from_stats(file: &str) -> Option<u64> {
    let re = Regex::new(r#"(inute"\s*:\s*)(\d+)"#).unwrap();

    for cap in re.captures_iter(file) {
        let stat = &cap[2];
        let out: u64 = stat.parse::<u64>().unwrap();

        return Some(out / 20);
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_to_time_format() {
        let x = convert_seconds_to_hh_mm_ss(127);
        let y = convert_seconds_to_hh_mm_ss(6666);

        assert_eq!(String::from("00:02:07"), x);
        assert_eq!(String::from("01:51:06"), y)
    }

    #[test]
    fn minecraft_folder_exists() {
        assert!(
            get_minecraft_folder_path().as_path().is_dir(),
            "Minecraft folder does not exists"
        );
    }

    /*
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
    */

    /*
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
    */

    #[test]
    fn player_ticks_from_stats() {
        let value = String::from(
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

        let expected = 119;

        assert_eq!(get_seconds_played_from_stats(&value), Some(expected));
    }
}
