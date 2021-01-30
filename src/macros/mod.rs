mod commands;
mod logging;

#[macro_export]
macro_rules! try_or_string_err {
    ($expr:expr, $val:expr) => {
        match $expr {
            std::result::Result::Ok(val) => val,
            std::result::Result::Err(_) => {
                return std::result::Result::Err($val);
            },
        }
    };
}
