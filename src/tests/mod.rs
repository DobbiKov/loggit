mod file_manager;
mod from_env_config;
mod from_env_file_config;
mod from_ini_file_config;
mod from_json_file_config;
use crate::Level;

use crate::helper;
use crate::logger::file_handler::file_formatter::FileFormatter;
use crate::logger::file_handler::file_manager::RotationType;
use crate::logger::file_handler::file_name::FileName;
use crate::logger::formatter::{parse_string_to_logparts, LogPart};
use crate::logger::init;
use crate::*;
use std::fs;

#[test]
fn parse_rotation_type() {
    let res = crate::logger::file_handler::file_manager::RotationType::try_from_string("dfsa week");
    assert_eq!(res, None);

    let res = crate::logger::file_handler::file_manager::RotationType::try_from_string("23 week");
    assert_eq!(
        res,
        Some(
            crate::logger::file_handler::file_manager::RotationType::Period(60 * 60 * 24 * 7 * 23)
        )
    )
}

#[test]
fn test_file_formatter_valid() {
    // Valid format must include a final text with an extension (e.g: ".txt" or ".log")
    let valid_format = "prefix_{time}_{date}.txt";
    let res = FileFormatter::try_from_string(valid_format);
    assert!(res.is_ok());
}

#[test]
fn test_file_formatter_forbidden_character() {
    // Format contains forbidden character '<'
    let invalid_format = "prefix_<{time}>.log";
    let res = FileFormatter::try_from_string(invalid_format);
    assert!(res.is_err());
}

#[test]
fn test_file_formatter_empty_string() {
    let res = FileFormatter::try_from_string("");
    assert!(res.is_err());
}

#[test]
fn test_file_formatter_no_extension() {
    let invalid_format = "prefix_{time}_{date}";
    let res = FileFormatter::try_from_string(invalid_format);
    assert!(res.is_err());
}

#[test]
fn test_file_name_from_formatter_success() {
    let format_str = "log_{date}_{time}.txt";
    let file_formatter = FileFormatter::try_from_string(format_str).unwrap();
    let file_name = FileName::from_file_formatter(file_formatter, Level::INFO);
    assert!(file_name.is_ok());
    let file_name = file_name.unwrap();
    let full_file_name: String = file_name.into();
    // Expect the file name to end with .txt
    assert!(full_file_name.ends_with(".txt"));
}

#[test]
fn test_file_name_from_formatter_incorrect_extension() {
    // Use an extension that is not in the allowed list
    let format_str = "log_{date}.csv";
    let file_formatter = FileFormatter::try_from_string(format_str);
    assert!(file_formatter.is_ok());
    let file_name = FileName::from_file_formatter(file_formatter.unwrap(), Level::DEBUG);
    assert!(file_name.is_err());
}

#[test]
fn test_helper_date_time() {
    let date_str = helper::get_current_date_in_string();
    let time_str = helper::get_current_time_in_string();
    // Check that the helper functions return non-empty strings
    assert!(!date_str.is_empty());
    assert!(!time_str.is_empty());
}

#[test]
fn test_seconds_to_ymdhms_epoch() {
    let (year, month, day, hour, minute, _) = helper::seconds_to_ymdhms(0);
    // Epoch time: 1970-01-01 00:00:00
    assert_eq!(year, 1970);
    assert_eq!(month, 1);
    assert_eq!(day, 1);
    assert_eq!(hour, 0);
    assert_eq!(minute, 0);
}

#[test]
fn test_parse_string_to_logparts() {
    let format_str = "<green>[{level}]<green> <blue>({file} {line})<blue> - {message}";
    let parts = parse_string_to_logparts(format_str).unwrap();
    // The returned vector should not be empty
    assert!(!parts.is_empty());
    // Expect at least one part corresponding to text (e.g., "[")
    assert!(parts
        .iter()
        .any(|p| matches!(p, LogPart::Text(t) if !t.is_empty())));
}

#[test]
fn test_log_macros_execution() {
    // Initialize logger with default configuration.
    init();
    // Set terminal output to true
    logger::set_print_to_terminal(true);

    // Simply call the macros. The test passes if nothing panics.
    trace!("Test trace message");
    debug!("Test debug message: {}", 123);
    info!("Test info message");
    warn!("Test warn message");
    error!("Test error message");
}

#[test]
fn test_rotation_type_parsing() {
    // Invalid rotation string
    let invalid = RotationType::try_from_string("invalid");
    assert!(invalid.is_none());

    // Test size rotation
    let size = RotationType::try_from_string("500 MB");
    assert!(size.is_some());

    // Test period rotation
    let period = RotationType::try_from_string("2 week");
    assert!(period.is_some());

    // Test time rotation
    let time = RotationType::try_from_string("12:30");
    assert!(time.is_some());
}

#[test]
fn test_set_file_and_compression_and_rotation() {
    // Initialize logger and configure file handling.
    init();
    logger::set_file("app_{date}_{time}.txt");
    logger::set_compression("zip");
    logger::add_rotation("1 day");

    // Check that the internal config now includes a file_manager.
    let config_state = CONFIG.read().unwrap();
    let cfg = config_state;
    assert!(cfg.file_manager.is_some());

    // Optionally, clean up any generated file if needed.
    let file_name = cfg
        .file_manager
        .as_ref()
        .unwrap()
        .lock()
        .unwrap()
        .get_file_name();

    if fs::metadata(&file_name).is_ok() {
        let _ = fs::remove_file(file_name);
    }
}
