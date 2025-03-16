//! # Loggit
//!
//! Loggit is a lightweight logging library for Rust that provides built‑in logger macros
//! (such as `trace!`, `debug!`, `info!`, `warn!`, and `error!`) to allow you to start logging
//! with zero boilerplate. You can also customize the log level, format, colors, and output destination.
//!
//! ## Features
//!
//! - **Zero Setup**: Just import the library and start logging immediately.
//! - **Customizable** logging levels and formats.
//! - **Colorized Output**: Optionally colorize messages based on log level.
//!
//! ## Installation
//!
//! Add the following to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! loggit = "0.1.0"
//! ```
//!
//! Or use:
//!
//! ```shell
//! cargo add loggit
//! ```
//!
//! ## Usage Example
//!
//! ```rust
//! use loggit::{init, trace, debug, info, warn, error, logger::set_log_level, Level};
//!
//! fn main() {
//!     // Optionally set a custom log level.
//!     set_log_level(Level::DEBUG);
//!
//!     trace!("This is a trace message.");
//!     debug!("Debug message: value = {}", 42);
//!     info!("Informational message.");
//!     warn!("Warning: check configuration!");
//!     error!("Error occurred: {}", "example error");
//! }
//! ```
//!
//! ## Modules
//!
//! - [`logger`]: Contains functions to control logging configuration and macros to log messages.
//! - [`helper`]: Provides time conversion utilities.
//! - [`logger::formatter`]: Contains the log format parser and formatter definitions.


use logger::formatter::LogFormatter;
use once_cell::sync::Lazy;
use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};
pub mod helper;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
/// Represents the log level used throughout the application.
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
