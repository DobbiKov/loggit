use colored::Colorize;
use formatter::{LogColor, LogFormatter};

use crate::{Config, Level, CONFIG};
//pub(crate) mod formatter;
pub mod formatter;

struct LogInfo {
    file: String,
    line: u32,
    message: String,
    level: Level,
}

//config getters
fn get_current_log_level() -> Level {
    let config_lock = CONFIG.read().unwrap();
    if let Some(ref cfg) = *config_lock {
        cfg.level
    } else {
        panic!("Something went wrong with getting log level!")
    }
}
fn get_config() -> Config {
    let config_lock = CONFIG.read().unwrap();
    if let Some(ref cfg) = *config_lock {
        cfg.clone()
    } else {
        panic!("Something went wrong with getting config!")
    }
}

fn get_log_format() -> LogFormatter {
    get_config().log_format
}

// config setters
pub fn set_log_level(lvl: Level) {
    let mut config_lock = CONFIG.write().unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.level = lvl;
    }
}
pub fn set_print_to_terminal(val: bool) {
    let mut config_lock = CONFIG.write().unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.print_to_terminal = val;
    }
}
pub fn set_colorized(val: bool) {
    let mut config_lock = CONFIG.write().unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.colorized = val;
    }
}
pub fn set_formatting(format: String) {
    let mut config_lock = CONFIG.write().unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.log_format = LogFormatter::parse_from_string(format);
    }
}

// funcs to log
fn string_log(log_info: &LogInfo) -> String {
    let mut mess_to_print = String::new();
    let curr_time = String::new();
    let curr_date = String::new();
    for log_part in get_log_format().parts {
        let str_to_push = match log_part.part {
            formatter::LogPart::Message => &log_info.message,
            formatter::LogPart::Time => &curr_time,
            formatter::LogPart::File => &log_info.file,
            formatter::LogPart::Line => &log_info.line.to_string(),
            formatter::LogPart::Date => &curr_date,
            formatter::LogPart::Level => &log_info.level.to_string(),
            formatter::LogPart::Text(text) => &text.clone(),
        };
        if get_config().colorized && log_part.color.is_some() {
            let colored_str = LogColor::colorize_str(str_to_push, log_part.color.unwrap());
            mess_to_print.push_str(&colored_str);
        } else {
            mess_to_print.push_str(str_to_push);
        }
    }
    mess_to_print
}
fn print_log(log_info: &LogInfo) {
    let mess_to_print = string_log(log_info);
    println!("{}", mess_to_print);
}
fn log_handler(log_info: LogInfo) {
    if get_config().print_to_terminal {
        print_log(&log_info);
    }
}
fn macro_handler(file: String, line: u32, deb_str: String, level: Level) {
    let log_info = LogInfo {
        file,
        line,
        message: deb_str,
        level,
    };
    if level >= get_current_log_level() {
        log_handler(log_info);
    }
}

pub fn __debug_handler(file: &str, line: u32, deb_str: String, level: Level) {
    macro_handler(file.to_string(), line, deb_str, level);
}

#[macro_export]
macro_rules! trace {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::TRACE);
        }};
    }

#[macro_export]
macro_rules! debug {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::DEBUG);
        }};
    }

#[macro_export]
macro_rules! info {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::INFO);
        }};
    }

#[macro_export]
macro_rules! warn {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::WARN);
        }};
    }

#[macro_export]
macro_rules! error {
        ($($arg:tt)*) => {{
            let res_str = format!($($arg)*);
            $crate::logger::__debug_handler(file!(), line!(), res_str, $crate::Level::ERROR);
        }};
    }

pub fn init() {
    let mut config = CONFIG.write().unwrap();
    *config = Some(Config {
        ..Default::default()
    })
}
