use std::{
    fs::create_dir,
    io::{Read, Write},
    path::Path,
};

use colored::Colorize;
use fern::{Dispatch, InitError};
use log::LevelFilter;

#[macro_export]
macro_rules! inori_success {
    ($e:tt, $($arg:tt)+) => (
        log::info!("[{}][{}] {}", "+".green().bold(), $e.bright_white(), format!($($arg)+));
    );
}

#[macro_export]
macro_rules! inori_debug {
    ($e:tt, $($arg:tt)+) => (
        log::debug!("[{}][{}] {}", "*".white().bold(), $e.bright_white(), format!($($arg)+));
    );
}

#[macro_export]
macro_rules! inori_info {
    ($e:tt, $($arg:tt)+) => (
        log::info!("[{}][{}] {}", "+".white().bold(), $e.bright_white(), format!($($arg)+));
    );
}

#[macro_export]
macro_rules! inori_warn {
    ($e:tt, $($arg:tt)+) => (
        log::warn!("[{}][{}] {}", "-".orange().bold(), $e.bright_white(), format!($($arg)+));
    );
}

#[macro_export]
macro_rules! inori_error {
    ($e:tt, $($arg:tt)+) => (
        log::error!("[{}][{}] {}", "!".red().bold(), $e.bright_white(), format!($($arg)+));
    );
}

#[macro_export]
macro_rules! inori_panic {
    ($e:tt, $($arg:tt)+) => (
        log::error!("[{}][{}] {}", "!!!".red().bold(), $e.bright_white(), format!($($arg)+));
        crate::utils::logging::exit();
    )
}

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