use std::io::{Read, Write};

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
    if cfg!(windows) {
        colored::control::set_override(false);
    }

    Dispatch::new()
        .format(move |out, message, _record| {
            out.finish(format_args!("[{}]{}", chrono::Local::now().format("%H:%M:%S"), message))
        })
        .level_for("serenity", LevelFilter::Off)
        .level_for("tracing::span", LevelFilter::Off)
        .level(LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;

    inori_success!("Log", "Successfully setup logger");

    Ok(())
}
