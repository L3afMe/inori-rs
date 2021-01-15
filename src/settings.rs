use std::{
    fs::File,
    io::{Read, Write},
};

use crate::models::settings::Settings;

pub fn load_settings() -> Settings {
    let mut f = File::open("config.toml")
        .expect("Unable to find the config.toml file. Copy 'config.toml.bak' to 'config.toml' and edit the file");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Unable to read 'config.toml'");
    let res = toml::from_str(&contents).expect("Unable to deserialize config");
    println!("[Config] Load successful");

    res
}

pub fn save_settings(settings: &Settings) {
    let contents = toml::to_string(settings).expect("Unable to serialize config");
    let mut f = File::create("config.toml").expect("Error creating 'config.toml' file");
    f.write_all(&contents.as_bytes())
        .expect("Unable to write config to file");
    f.sync_data().expect("Unable to write config file to disk");
    println!("[Config] Save successful");
}
