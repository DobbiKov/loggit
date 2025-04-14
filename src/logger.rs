//! The logger module provides configuration functions and macros for logging.
//!
//! You can change the log level, formatting, and enable/disable colorized output.
//!
//! The public macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`) use the internal
//! handlers to format and print the log message.

use file_handler::file_manager::FileManager;
use formatter::{LogColor, LogFormatter};
use set_errors::{
    AddRotationError, SetColorizedError, SetCompressionError, SetFileError,
    SetLevelFormattingError, SetLogLevelError, SetPrintToTerminalError,
};
use std::sync::RwLockWriteGuard;

use crate::{
    helper::{get_current_date_in_string, get_current_time_in_string},
    Config, Level, CONFIG,
};
//pub(crate) mod formatter;
pub mod file_handler;
pub mod formatter;
pub mod from_file_config;
pub mod set_errors;

struct LogInfo {
    module_path: String,
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

fn get_file_manager() -> Option<FileManager> {
    let tmp_cfg = get_config();
    tmp_cfg.file_manager
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
//

/// Makes all logs to be saved to files. The files take names accordinlgy to the provided format.
///
///  Configures Loggit to write logs to a file. The `format` string is used to generate the file name and must include a file extension. The format may include placeholders such as:
///  - `{time}` – Current time.
///  - `{date}` – Current date.
///  - `{level}` - Current loggin level.
///  - Other literal text.
///
///- **Allowed values:**  
///  - The format string **must** end with a text section containing a file extension (e.g. `.txt` or `.log`).  
///  - Any forbidden characters such as `<`, `>`, `&`, or `%` will cause configuration to fail.  
///  - *Examples:*  
///    - `"app_{date}_{time}.txt"`  
///    - `"{level}-log-on-{date}.log"`
pub fn set_file(format: &str) -> Result<(), SetFileError> {
    let file_manager = match FileManager::init_from_string(format, get_config()) {
        Ok(r) => r,
        Err(e) => {
            return Err(SetFileError::UnableToLoadFromString(e));
        }
    };

    let config_lock = get_write_config();
    if config_lock.is_none() {
        return Err(SetFileError::UnableToLoadConfig);
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.file_manager = Some(file_manager);
    }
    Ok(())
}

///Enables file compression for log archival.
///
///- **Description:**  
///  Sets the compression type for log files. After file logging is configured, you can enable compression to archive old logs.
///
///- **Allowed values:**  
///  - Accepts only a single allowed value: `"zip"`.  
///  - Any other string will output an error and leave the compression configuration unchanged.
pub fn set_compression(ctype: &str) -> Result<(), SetCompressionError> {
    let f_manager = get_file_manager();
    if f_manager.is_none() {
        return Err(SetCompressionError::FileIsntSet);
    }
    let mut f_manager = f_manager.unwrap();
    f_manager.set_compression(ctype);

    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return Err(SetCompressionError::UnableToLoadConfig);
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.file_manager = Some(f_manager);
    }
    Ok(())
}

///Adds a new constraint for rotating log files.
///
///- **Description:**  
///  Adds a rotation strategy so that log files are rotated based on either time or file size. When a log file “expires” under the configured constraint, a new file is automatically created (and optionally compressed).
///
///- **Allowed values:**  
///  The `constraint` string can be in one of the following formats:
///  - **Period rotation:**  
///    - Numeric value followed by a unit:  
///      - `"1 hour"`, `"2 day"`, `"33 week"`, `"6 month"`, `"12 year"`  
///      - The unit is case sensitive and must match exactly (e.g. `" hour"`, `" day"`, etc.).
///  - **Time-based rotation:**  
///    - Time in a 24‑hour format using a colon separator:  
///      - `"HH:MM"` (e.g. `"12:30"`).
///  - **Size-based rotation:**  
///    - Numeric value followed by a size unit:  
///      - `"500 KB"`, `"5 MB"`, `"1 GB"`, or `"2 TB"`  
///      - Note the space before the unit.
///
///- If an incorrect value is provided, the rotation is not added and an error message is logged.
pub fn add_rotation(constraint: &str) -> Result<(), AddRotationError> {
    let f_manager = get_file_manager();
    if f_manager.is_none() {
        eprintln!("Can't add a rotation when the file isn't set!");
        return Err(AddRotationError::FileIsntSet);
    }
    let mut f_manager = f_manager.unwrap();

    if !f_manager.add_rotation(constraint) {
        eprintln!("Incorrect value given for the rotation!");
        return Err(AddRotationError::IncorrectFormatGiven);
    }

    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return Err(AddRotationError::UnableToLoadConfig);
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.file_manager = Some(f_manager);
    }
    Ok(())
}

/// Sets the minimum log level to display.
/// Messages with a level lower than the given level will be ignored.
pub fn set_log_level(lvl: Level) -> Result<(), SetLogLevelError> {
    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return Err(SetLogLevelError::UnableToLoadConfig);
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.level = lvl;
    }
    Ok(())
}
/// Enables or disables terminal output of log messages.
/// When set to false, log messages will not be printed to the terminal.
pub fn set_print_to_terminal(val: bool) -> Result<(), SetPrintToTerminalError> {
    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return Err(SetPrintToTerminalError::UnableToLoadConfig);
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.print_to_terminal = val;
    }
    Ok(())
}
/// Enables or disables colorized output of log messages.
/// If enabled, logs will be printed with colors as configured in the format.
pub fn set_colorized(val: bool) -> Result<(), SetColorizedError> {
    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return Err(SetColorizedError::UnableToLoadConfig);
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        cfg.colorized = val;
    }
    Ok(())
}

/// Sets a global log formatting string for all log levels.
/// This function updates the formatting of each level to the given template.
pub fn set_global_formatting(format: &str) -> Result<(), SetLevelFormattingError> {
    set_level_formatting(Level::TRACE, format)?;
    set_level_formatting(Level::DEBUG, format)?;
    set_level_formatting(Level::INFO, format)?;
    set_level_formatting(Level::WARN, format)?;
    set_level_formatting(Level::ERROR, format)?;
    Ok(())
}

/// Sets a custom log formatting string for the specified log level.
///
/// The formatting string may contain placeholders like `{level}`, `{file}`, `{line}`, `module` and `{message}`.
///
/// Example:
/// ```rust
/// use loggit::logger;
/// use loggit::Level;
///
/// fn main(){
///     logger::set_level_formatting(Level::DEBUG, "{module} <red>{level}<red> [{file}] {message}");
/// }
/// ```
pub fn set_level_formatting(level: Level, format: &str) -> Result<(), SetLevelFormattingError> {
    let config_lock = get_write_config();
    if config_lock.is_none() {
        eprintln!("An error while getting the config to write!");
        return Err(SetLevelFormattingError::UnableToLoadConfig);
    }
    let mut config_lock = config_lock.unwrap();
    if let Some(ref mut cfg) = *config_lock {
        match level {
            Level::TRACE => cfg.trace_log_format = LogFormatter::parse_from_string(format)?,
            Level::DEBUG => cfg.debug_log_format = LogFormatter::parse_from_string(format)?,
            Level::INFO => cfg.info_log_format = LogFormatter::parse_from_string(format)?,
            Level::WARN => cfg.warn_log_format = LogFormatter::parse_from_string(format)?,
            Level::ERROR => cfg.error_log_format = LogFormatter::parse_from_string(format)?,
        }
    }
    Ok(())
}

// -- Internal functions for logging --
fn string_log(log_info: &LogInfo, colorize: bool) -> String {
    let mut mess_to_print = String::new();
    let curr_time: String = get_current_time_in_string();
    let curr_date = get_current_date_in_string();
    for log_part in get_log_format(log_info.level).parts {
        let str_to_push = match log_part.part {
            formatter::LogPart::Message => &log_info.message,
            formatter::LogPart::Time => &curr_time,
            formatter::LogPart::File => &log_info.file,
            formatter::LogPart::Line => &log_info.line.to_string(),
            formatter::LogPart::Date => &curr_date,
            formatter::LogPart::Level => &log_info.level.to_string(),
            formatter::LogPart::Text(text) => &text.clone(),
            formatter::LogPart::ModulePath => &log_info.module_path,
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
    let mut file_manager = get_file_manager().unwrap();
    let mess_to_print = string_log(log_info, false);

    let res = file_manager.write_log(&mess_to_print, get_config());
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "Couldn't write a log to the file due to the next error: {}",
                e
            );
        }
    }

    // file manager can be updated, (for example change of file) thus it needs to be updated in the
    // config
    let w_conf = get_write_config();
    if let Some(mut temp_cfg) = w_conf {
        if let Some(ref mut cfg) = *temp_cfg {
            cfg.file_manager = Some(file_manager)
        }
    }
}
fn log_handler(log_info: LogInfo) {
    if get_config().print_to_terminal {
        print_log(&log_info);
    }
    if get_config().file_manager.is_some() {
        write_file_log(&log_info);
    }
}

// handles call from macro and passes deeper
fn macro_handler(module_path: &str, file: &str, line: u32, deb_str: String, level: Level) {
    let log_info = LogInfo {
        module_path: module_path.to_string(),
        file: file.to_string(),
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
pub fn __debug_handler(module_path: &str, file: &str, line: u32, deb_str: String, level: Level) {
    macro_handler(module_path, file, line, deb_str, level);
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
            $crate::logger::__debug_handler(module_path!(), file!(), line!(), res_str, $crate::Level::TRACE);
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
            $crate::logger::__debug_handler(module_path!(), file!(), line!(), res_str, $crate::Level::DEBUG);
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
            $crate::logger::__debug_handler(module_path!(), file!(), line!(), res_str, $crate::Level::INFO);
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
            $crate::logger::__debug_handler(module_path!(), file!(), line!(), res_str, $crate::Level::WARN);
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
            $crate::logger::__debug_handler(module_path!(), file!(), line!(), res_str, $crate::Level::ERROR);
        }};
    }

/// Initializes the logger with default configuration settings.
pub fn init() {
    let mut config = CONFIG.write().unwrap();
    *config = Some(Config {
        ..Default::default()
    })
}
