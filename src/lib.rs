use logger::formatter::LogFormatter;
use once_cell::sync::Lazy;
use std::{
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
    trace_log_format: LogFormatter,
    debug_log_format: LogFormatter,
    info_log_format: LogFormatter,
    warn_log_format: LogFormatter,
    error_log_format: LogFormatter,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: Default::default(),
            print_to_terminal: true,
            colorized: false,
            trace_log_format: Default::default(),
            debug_log_format: Default::default(),
            info_log_format: Default::default(),
            warn_log_format: Default::default(),
            error_log_format: LogFormatter::parse_from_string(
                "<red>[{level}]<red> <blue>({file} {line})<blue> - <red>{message}<red>".to_string(),
            ),
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
