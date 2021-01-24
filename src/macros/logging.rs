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
