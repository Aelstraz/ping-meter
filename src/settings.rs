use std::{env, fs};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub ping_interval: f32,
    pub ping_timeout: f32,
    pub ping_address: String,
    pub max_plot_points: usize,
    pub plot_aspect_ratio: f32,
    #[serde(skip_serializing)]
    pub update: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            ping_interval: 1.0,
            ping_timeout: 3.0,
            ping_address: String::from("8.8.8.8"),
            max_plot_points: 30,
            plot_aspect_ratio: 1.0,
            update: false,
        }
    }
}

impl Settings {
    const FILE_NAME: &str = "settings.json";

    pub fn save(&mut self) {
        match serde_json::to_string(self) {
            Ok(val) => match env::current_dir() {
                Ok(path) => match fs::write(path.join(Self::FILE_NAME), val) {
                    Ok(_) => {}
                    Err(err) => eprintln!("Failed writing settings file: {}", err),
                },
                Err(err) => eprintln!("Failed getting current directory: {}", err),
            },
            Err(err) => eprintln!("Failed to serialize: {}", err),
        }
        self.update = true;
    }

    pub fn load() -> Self {
        match env::current_dir() {
            Ok(path) => match fs::read_to_string(path.join(Self::FILE_NAME)) {
                Ok(val) => {
                    return serde_json::from_str::<Settings>(&val).unwrap_or_default();
                }
                Err(err) => eprintln!("Failed reading settings file: {}", err),
            },
            Err(err) => eprintln!("Failed getting current directory: {}", err),
        }
        return Self::default();
    }
}
