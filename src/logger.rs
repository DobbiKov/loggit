//! The logger module provides configuration functions and macros for logging.
//!
//! You can change the log level, formatting, and enable/disable colorized output.
//!
//! The public macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`) use the internal
//! handlers to format and print the log message.

use file_handler::FileFormatter;
use formatter::{LogColor, LogFormatter};
use std::{
    sync::RwLockWriteGuard,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{helper, Config, FileConfig, Level, CONFIG};
//pub(crate) mod formatter;
pub mod file_handler;
pub mod formatter;

struct LogInfo {
    file: String,
    line: u32,
    message: String,
    level: Level,
}

// -- Getter functions for config --
fn get_log_level() -> Level {
    get_config().level
}
fn get_config() -> Config {
    let config_lock = match CONFIG.read() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Problem with getting config, here's an error: {}", e);
            return Default::default();
        }
    };
    if let Some(ref cfg) = *config_lock {
        cfg.clone()
    } else {
        eprintln!("Problem with getting config!");
        Default::default()
    }
}

fn get_log_format(level: Level) -> LogFormatter {
    let tmp_cfg = get_config();
    match level {
        Level::TRACE => tmp_cfg.trace_log_format,
        Level::DEBUG => tmp_cfg.debug_log_format,
        Level::INFO => tmp_cfg.info_log_format,
        Level::WARN => tmp_cfg.warn_log_format,
        Level::ERROR => tmp_cfg.error_log_format,
    }
}

fn get_file_config() -> Option<FileConfig> {
    let tmp_cfg = get_config();
    tmp_cfg.file_config
}

fn get_write_config() -> Option<RwLockWriteGuard<'static, Option<Config>>> {
    match CONFIG.write() {
        Ok(guard) => Some(guard),
        Err(e) => {
            eprintln!(
                "An error while getting the config to write, here's an error: {}",
                e
            );
            None
        } // Handle error case safely
    }
}

// -- Public configuration setter functions --
pub fn set_file(format: String) {
    let file_format = FileFormatter::from_string(format);
    let file_name = file_format.get_file_name(get_log_level());

    let file_config = FileConfig {
        file_format,
        current_file_name: file_name,
    };

    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return;
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.file_config = Some(file_config);
    }
}

/// Sets the minimum log level to display.
/// Messages with a level lower than the given level will be ignored.
pub fn set_log_level(lvl: Level) {
    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return;
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.level = lvl;
    }
}
/// Enables or disables terminal output of log messages.
/// When set to false, log messages will not be printed to the terminal.
pub fn set_print_to_terminal(val: bool) {
    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return;
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.print_to_terminal = val;
    }
}
/// Enables or disables colorized output of log messages.
/// If enabled, logs will be printed with colors as configured in the format.
pub fn set_colorized(val: bool) {
    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return;
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.colorized = val;
    }
}

/// Sets a global log formatting string for all log levels.
/// This function updates the formatting of each level to the given template.
pub fn set_global_formatting(format: String) {
    set_level_formatting(Level::TRACE, format.clone());
    set_level_formatting(Level::DEBUG, format.clone());
    set_level_formatting(Level::INFO, format.clone());
    set_level_formatting(Level::WARN, format.clone());
    set_level_formatting(Level::ERROR, format);
}

/// Sets a custom log formatting string for the specified log level.
///
/// The formatting string may contain placeholders like `{level}`, `{file}`, `{line}`, and `{message}`.
pub fn set_level_formatting(level: Level, format: String) {
    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return;
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        match level {
            Level::TRACE => cfg.trace_log_format = LogFormatter::parse_from_string(format.clone()),
            Level::DEBUG => cfg.debug_log_format = LogFormatter::parse_from_string(format.clone()),
            Level::INFO => cfg.info_log_format = LogFormatter::parse_from_string(format.clone()),
            Level::WARN => cfg.warn_log_format = LogFormatter::parse_from_string(format.clone()),
            Level::ERROR => cfg.error_log_format = LogFormatter::parse_from_string(format.clone()),
        }
    }
}

// -- Internal functions for logging --
fn string_log(log_info: &LogInfo, colorize: bool) -> String {
    let mut mess_to_print = String::new();
    let ymdhms = crate::helper::seconds_to_ymdhms(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );
    let curr_time: String = format!("{}:{}:{}", ymdhms.3, ymdhms.4, ymdhms.5);
    let curr_date = format!("{}:{}:{}", ymdhms.2, ymdhms.1, ymdhms.0);
    for log_part in get_log_format(log_info.level).parts {
        let str_to_push = match log_part.part {
            formatter::LogPart::Message => &log_info.message,
            formatter::LogPart::Time => &curr_time,
            formatter::LogPart::File => &log_info.file,
            formatter::LogPart::Line => &log_info.line.to_string(),
            formatter::LogPart::Date => &curr_date,
            formatter::LogPart::Level => &log_info.level.to_string(),
            formatter::LogPart::Text(text) => &text.clone(),
        };
        if colorize && log_part.color.is_some() {
            let colored_str = LogColor::colorize_str(str_to_push, log_part.color.unwrap());
            mess_to_print.push_str(&colored_str);
        } else {
            mess_to_print.push_str(str_to_push);
        }
    }
    mess_to_print
}
fn print_log(log_info: &LogInfo) {
    let mess_to_print = string_log(log_info, get_config().colorized);
    println!("{}", mess_to_print);
}
fn write_file_log(log_info: &LogInfo) {
    let file_config = get_file_config().unwrap();
    let mess_to_print = string_log(log_info, false);

    // TODO: here_ve_must_verify_the_time_and_others constraints
    match helper::write_to_file(&file_config.current_file_name, &mess_to_print) {
        Ok(()) => {}
        Err(_) => {
            println!(
                "SOMETHING WENT WRONG WHILE TRYING TO WRITE TO A FILE! {} | {}",
                file_config.current_file_name, mess_to_print
            );
        }
    }
}
fn log_handler(log_info: LogInfo) {
    if get_config().print_to_terminal {
        print_log(&log_info);
    }
    if get_config().file_config.is_some() {
        write_file_log(&log_info);
    }
}

// handles call from macro and passes deeper
fn macro_handler(file: String, line: u32, deb_str: String, level: Level) {
    let log_info = LogInfo {
        file,
        line,
        message: deb_str,
        level,
    };
    if level >= get_log_level() {
        log_handler(log_info);
    }
}

/// Internal function for handling log macros.
///
/// It is used by the public logger macros to format and output the log message.
pub fn __debug_handler(file: &str, line: u32, deb_str: String, level: Level) {
    macro_handler(file.to_string(), line, deb_str, level);
}

// -- Publicly exported logging macros --

#[macro_export]
/// Logs a message at the TRACE level.
/// The message is formatted using standard Rust formatting.
///
/// # Example
/// ```rust
/// use loggit::trace;
///
/// trace!("Trace message: {}", "details");
/// ```
macro_rules! trace {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::TRACE);
        }};
    }

#[macro_export]
/// Logs a message at the DEBUG level.
/// The message is formatted using standard Rust formatting.
///
/// # Example
/// ```rust
/// use loggit::debug;
///
/// debug!("Debug message: value = {}", 123);
/// ```
macro_rules! debug {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::DEBUG);
        }};
    }

#[macro_export]
/// Logs a message at the INFO level.
/// The message is formatted using standard Rust formatting.
///
/// # Example
/// ```rust
/// use loggit::info;
///
/// info!("Informational message.");
/// ```
macro_rules! info {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::INFO);
        }};
    }

#[macro_export]
/// Logs a message at the WARN level.
/// The message is formatted using standard Rust formatting.
///
/// # Example
/// ```rust
/// use loggit::warn;
///
/// warn!("Warning: check configuration!");
/// ```
macro_rules! warn {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::WARN);
        }};
    }

#[macro_export]
/// Logs a message at the ERROR level.
/// The message is formatted using standard Rust formatting.
///
/// # Example
/// ```rust
/// use loggit::error;
///
/// error!("Error occurred: {}", "example error");
/// ```
macro_rules! error {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::ERROR);
        }};
    }

/// Initializes the logger with default configuration settings.
pub fn init() {
    let mut config = CONFIG.write().unwrap();
    *config = Some(Config {
        ..Default::default()
    })
}
