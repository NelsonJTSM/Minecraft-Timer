extern crate dirs;

use std::path::{Path, PathBuf};

pub fn run() {
    println!("Hello, world!");
}

fn get_minecraft_folder_path() -> PathBuf {
    dirs::home_dir().expect("Could not get home directory").join(".minecraft")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minecraft_folder_exists() {
        assert!(get_minecraft_folder_path().as_path().is_dir(), "Minecraft folder does not exists");
    }
}