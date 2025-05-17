//! # Loggit
//!
//! Loggit is a lightweight logging library for Rust that provides built‑in logger macros
//! (such as `trace!`, `debug!`, `info!`, `warn!`, and `error!`) to allow you to start logging
//! with zero boilerplate. You can also customize the log level, format, colors, and output destination.
//!
//! ## Features
//!
//!- **Zero Setup**: Just import the library and start logging.
//!- **Customizable**: Change log formats, colors, and logging levels.
//!- **Macros Provided**: Includes `trace!`, `debug!`, `info!`, `warn!`, and `error!`.
//!- **Flexible Formatting**: Use custom templates with placeholders like `{level}`, `{file}`, `{line}`, and `{message}`.
//!- **Saving log to files**: Save your logs to files automaticaly by specifying filename format
//!- **File rotation**: Rotate your files by specifying time period or size
//!- **Compress used files**: Save your space by compressing used log files
//!
//! ## Installation
//!
//! Add the following to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! loggit = "0.1.6"
//! ```
//!
//! Or use:
//!
//! ```shell
//! cargo add loggit
//! ```
//!
//! ## Usage Examples
//!
//! ### Basic Logging
//!
//! Simply import the logger macros and use it in your project:
//!
//! ````rust
//! use loggit::{trace, debug, info, warn, error};
//!
//! fn main() {
//!     trace!("This is a trace message.");
//!     debug!("Debug message: variable value = {}", 42);
//!     info!("Informational message.");
//!     warn!("Warning: something might be off.");
//!     error!("Error occurred: {}", "example error");
//! }
//! ````
//!
//! ### Customizing the Log Level
//!
//! Set the minimum log level so that only messages at that level and above are printed:
//!
//! ````rust
//! use loggit::logger::set_log_level;
//! use loggit::Level;
//!
//! fn main() {
//!     // Set log level to DEBUG; TRACE messages will be ignored.
//!     set_log_level(Level::DEBUG);
//!
//!     debug!("This is a debug message.");
//!     trace!("This trace message will not be logged.");
//! }
//! ````
//!
//! ### Customizing the Log Format
//!
//! You can adjust the log format globally or per log level. Templates can include placeholders like `{level}`, `{file}`, `{line}`, and `{message}`. Colors can be configured by wrapping text with color tags.
//!
//! **Global Format Customization**
//!
//! ````rust
//! use loggit::logger::set_global_formatting;
//!
//! fn main() {
//!     // Set a global custom log format using color tags.
//!     set_global_formatting("<green>[{level}]<green> ({file}:{line}) - {message}");
//!
//!     info!("This info message follows the new global format.");
//!     info!("The error message as well.");
//! }
//! ````
//!
//! **Level-Specific Format Customization**
//!
//! ````rust
//! use loggit::logger::set_level_formatting;
//! use loggit::Level;
//!
//! fn main() {
//!     // Customize the ERROR log format specifically.
//!     set_level_formatting(
//!         Level::ERROR,
//!         "<red>[{level}]<red> <blue>({file}:{line})<blue> - <red>{message}<red>"
//!     );
//!
//!     error!("This error message will follow the custom error format.");
//! }
//! ````
//!
//! ### Enabling Colorized Output
//!
//! Enable or disable colored output based on your preference:
//!
//! ````rust
//! use loggit::logger::set_colorized;
//!
//! fn main() {
//!     // Enable colored output.
//!     set_colorized(true);
//!     
//!     info!("This info message will be colorized as specified in the format.");
//! }
//! ````
//!
//! ### Customizing Terminal Output
//!
//! Control whether messages are printed directly to the terminal:
//!
//! ````rust
//! use loggit::logger::set_print_to_terminal;
//!
//! fn main() {
//!     // Disable terminal output (for example, if you want to log to a file instead).
//!     set_print_to_terminal(false);
//!     
//!     info!("This message will not be printed to the terminal.");
//! }
//! ````
//!
//! ### Setting up logging to the file
//!
//! Enable save all your logs to a file
//!
//! ````rust
//! use loggit::logger::set_file;
//!
//! fn main() {
//!     // provide file name
//!     set_file("file_name.txt");
//! }
//! ````
//!
//! You can choose a format for the file name:
//!
//! ````rust
//! use loggit::logger::set_file;
//!
//! fn main() {
//!     // provide file name
//!     set_file("{level}-log-on-{date}.txt");
//! }
//! ````
//!
//! Choose how oftenly you change your file
//!
//! ````rust
//! use loggit::logger::{set_file, add_rotation};
//!
//! fn main() {
//!     // provide file name
//!     set_file("{level}-log-on-{date}.txt");
//!     add_rotation("1 week"); // change the file every week
//!     add_rotation("5 MB"); // max file size 5 MB, then again change of the file
//! }
//! ````
//!
//! Save your space by compressing log files
//! ```rust
//! use loggit::logger::{set_file, set_compression};
//!
//! fn main() {
//!     // provide file name
//!     set_file("{level}-log-on-{date}.txt");
//!     set_compression("zip");
//! }
//! ```
//!
//! Choose the directory to save archived log files to
//! ```rust
//! use loggit::logger::{set_file, set_compression, set_archive_dir};
//!
//! fn main() {
//!     // provide file name
//!     set_file("{level}-log-on-{date}.txt");
//!     set_compression("zip");
//!     set_archive_dir("my_archives"); // all the archives will be stored in the `my_archives` directory
//! }
//! ```
//! ### Configurate logger using env variables
//! ```sh
//! colorized=false file_name="save_here.txt" cargo run
//! ```
//!
//! ### Importing config from files
//! ```rust
//! use loggit::logger::{load_config_from_file};
//!
//! fn main(){
//!    let _ = load_config_from_file("my_conf.json");
//! }
//! ```
//!
//! Or simply crate a config file with one of those names:
//! - `loggit.env`
//! - `loggit.ini`
//! - `loggit.json`
//!
//! And it will be loaded automatically
//!
//! ## Modules
//!
//! - [`logger`]: Contains functions to control logging configuration and macros to log messages.

use logger::{file_handler::file_manager::FileManager, formatter::LogFormatter};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::{fmt::Display, path::PathBuf, sync::RwLock};
pub(crate) mod helper;

#[cfg(test)]
mod tests;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
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
    file_manager: Option<Arc<Mutex<FileManager>>>,
    archive_dir: Option<PathBuf>,
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
                "<red>[{level}]<red> <blue>({file} {line})<blue> - <red>{message}<red>",
            )
            .unwrap(),
            file_manager: None,
            archive_dir: None,
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

static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| {
    RwLock::new(Config {
        ..Default::default()
    })
});

pub mod logger;
