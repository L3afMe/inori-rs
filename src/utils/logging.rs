use std::{
    fs::create_dir,
    io::{Read, Write},
    path::Path,
};

use colored::Colorize;
use fern::{Dispatch, InitError};
use log::LevelFilter;

use crate::inori_success;

pub fn exit() {
    let mut stdout = std::io::stdout();
    write!(stdout, "Press enter to exit...").unwrap();
    stdout.flush().unwrap();
    let _ = std::io::stdin().read(&mut [0u8]).unwrap();
    std::process::exit(1);
}

pub fn setup_logger() -> Result<(), InitError> {
    let log_path = Path::new("logs");
    if !log_path.exists() {
        if let Err(why) = create_dir(log_path) {
            panic!("Unable to create log/ directory!\nError: {}", why);
        }
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    Dispatch::new()
        .format(move |out, message, _record| {
            out.finish(format_args!("[{}]{}", chrono::Local::now().format("%H:%M:%S"), message))
        })
        .level_for("serenity", LevelFilter::Off)
        .level_for("tracing::span", LevelFilter::Off)
        .level(LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!("logs/{}.log", now))?)
        .apply()?;

    inori_success!("Log", "Successfully setup logger");

    Ok(())
}
