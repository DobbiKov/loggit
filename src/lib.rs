use logger::formatter::LogFormatter;
use once_cell::sync::Lazy;
use std::{
    default,
    fmt::Display,
    sync::{Arc, RwLock},
};
pub mod helper;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Level {
    TRACE,
    DEBUG,
    #[default]
    INFO,
    WARN,
    ERROR,
}

#[derive(Clone)]
struct Config {
    level: Level,
    print_to_terminal: bool,
    colorized: bool,
    log_format: LogFormatter,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: Default::default(),
            print_to_terminal: true,
            colorized: false,
            log_format: Default::default(),
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_res = match self {
            Level::TRACE => "TRACE",
            Level::DEBUG => "DEBUG",
            Level::INFO => "INFO",
            Level::WARN => "WARN",
            Level::ERROR => "ERROR",
        };
        f.write_str(str_res)
    }
}

static CONFIG: Lazy<RwLock<Option<Config>>> = Lazy::new(|| {
    RwLock::new(Some(Config {
        ..Default::default()
    }))
});

pub mod logger;
