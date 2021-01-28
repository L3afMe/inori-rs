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
    if cfg!(windows) {
        colored::control::set_override(false);
    }

    let log = Dispatch::new()
        .format(move |out, message, _record| {
            out.finish(format_args!("[{}]{}", chrono::Local::now().format("%H:%M:%S"), message))
        })
        .level_for("serenity", LevelFilter::Off)
        .level_for("tracing::span", LevelFilter::Off)
        .level(LevelFilter::Info)
        .chain(std::io::stdout());

    let logs_path = Path::new(".").join("logs");
    if !logs_path.exists() {
        if let Err(why) = create_dir(logs_path.clone()) {
            println!("[WARNING] Unable to create log directory\n[ERROR] {:?}", why);

            log.apply()?;
            inori_success!("Log", "Successfully setup logger without log saving");
            return Ok(());
        }
    }

    let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_path = logs_path.join(format!("inori-rs_{}.log", now));

    match fern::log_file(log_path) {
        Ok(file) => {
            log.chain(file).apply()?;
            inori_success!("Log", "Successfully setup logger");
        },
        Err(why) => {
            println!("[WARNING] Unable to create log file!\n[ERROR] {:?}", why);
            log.apply()?;
            inori_success!("Log", "Successfully setup logger without log saving");
        },
    };

    Ok(())
}
