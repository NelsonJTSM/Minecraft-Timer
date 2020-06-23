extern crate dirs;
extern crate notify;

use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, SystemTime};

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;

pub struct Message {
    pub player: Option<Player>,
    pub world: Option<World>,
}

pub struct Player {
    pub ticks_played: u64,
}

pub struct World {
    pub last_modified: SystemTime,
}

// This function creates a new thread, that watches over a minecraft saves folder,
// then sends a Message to time_sender with any update stats.
// As of right now, it only works for Linux, since it looks for .minecraft
// on your home folder.
// TODO: Add default Windows and MacOS .minecraft location
// TODO: Add custom .minecraft location
pub fn run(time_sender: mpsc::Sender<Message>) {
    let minecraft_folder = get_minecraft_folder_path();
    let saves_folder = minecraft_folder.join("saves");

    // Creates a thread that continously looks for new stats files on .minecraft
    // and sends it as a Message to time_sender.
    let _stat_thread = thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1)).unwrap();
        watcher
            .watch(saves_folder, RecursiveMode::Recursive)
            .unwrap();

        loop {
            match rx.recv() {
                Ok(event) => {
                    match event {
                        DebouncedEvent::Write(file) => {
                            if is_stat_file(&file) {
                                let last_modified = file.metadata().unwrap().modified().unwrap();
                                let ticks_played = get_ticks_played_from_stats(
                                    &fs::read_to_string(&file).unwrap(),
                                )
                                .unwrap();

                                let player = Some(Player { ticks_played });
                                let world = Some(World { last_modified });

                                time_sender.send(Message { player, world }).unwrap();
                            }
                        }
                        // Assumes this is the first time a world is being loaded up.
                        // Written so the timer starts at 00:00
                        DebouncedEvent::Create(file) => {
                            if is_new_world(&file) {
                                let last_modified = file.metadata().unwrap().modified().unwrap();

                                let world = Some(World { last_modified });
                                time_sender
                                    .send(Message {
                                        world,
                                        player: None,
                                    })
                                    .unwrap();
                            }
                        }
                        _ => (),
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }

            thread::sleep(Duration::from_millis(50));
        }
    });
}

// Returns true if the file is a file for a new minecraft world.
// TODO: add testing.
fn is_new_world(file: &PathBuf) -> bool {
    file.parent().unwrap().file_name().unwrap() == "saves"
}

fn is_stat_file(file: &PathBuf) -> bool {
    if file.extension().is_none()
        || file.parent().is_none()
        || file.parent().unwrap().file_name().is_none()
    {
        false
    } else {
        file.extension().unwrap() == "json"
            && file.parent().unwrap().file_name().unwrap() == "stats"
    }
}

pub fn get_time_difference(time: SystemTime, ticks: u64) -> String {
    let x = SystemTime::now().duration_since(time).unwrap().as_secs() + (ticks / 20);

    convert_seconds_to_hh_mm_ss(x)
}

pub fn convert_seconds_to_hh_mm_ss(time: u64) -> String {
    let mut time_left = time;
    let hours = time_left / 3600;
    time_left -= hours * 3600;
    let minutes = time_left / 60;
    time_left -= minutes * 60;
    let seconds = time_left;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn get_minecraft_folder_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not get home directory")
        .join(".minecraft")
}

// Uses a regex to grab the playedOneMinute statistic.
// Works for both Minecraft 1.7.2 and 1.15
// 20 ticks = 1 second
// TODO: Refractor this code, bad stuff in it.
fn get_ticks_played_from_stats(file: &str) -> Option<u64> {
    let re = Regex::new(r#"(inute"\s*:\s*)(\d+)"#).unwrap();

    for cap in re.captures_iter(file) {
        let stat = &cap[2];
        let out: u64 = stat.parse::<u64>().unwrap();

        return Some(out);
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

        assert_eq!(get_ticks_played_from_stats(&value), Some(expected));
    }
}
