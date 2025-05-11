#![allow(unused_imports)] // temp for dev
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    logger::{
        init, load_config_from_file,
        set_errors::{
            AddRotationError, ReadFromConfigFileError, SetArchiveDirError, SetColorizedError,
            SetCompressionError, SetFileError, SetLevelFormattingError, SetLogLevelError,
            SetPrintToTerminalError,
        },
    },
    Level, CONFIG,
};
// Assuming Config is accessible as crate::Config because this file is in src/tests/
use crate::logger::formatter::LogFormatter;
use crate::Config as LoggerConfig;

// Helper to create a temporary JSON file
fn temp_json_file(contents: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_name = format!("loggit_json_test_{}.json", ts);
    path.push(file_name);

    let mut file = File::create(&path)
        .unwrap_or_else(|e| panic!("Failed to create temp json-file at {:?}: {}", path, e));
    write!(file, "{}", contents)
        .unwrap_or_else(|e| panic!("Failed to write temp json-file at {:?}: {}", path, e));
    path
}

// Helper to get a snapshot of the config
fn config_snapshot() -> LoggerConfig {
    CONFIG.read().expect("CONFIG should be readable").clone()
}

// Teardown for archive directories
fn cleanup_archive_dir(path_str: &str) {
    let path = Path::new(path_str);
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(path).ok();
    }
}

#[test]
fn json_enabled_variants() {
    // enabled=false → DisabledToBeUsed
    init();
    let p = temp_json_file(r#"{"enabled": "false"}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::DisabledToBeUsed)
    ));
    fs::remove_file(p).ok();

    // enabled missing (not set) should simply work (Ok)
    init();
    let p = temp_json_file(r#"{"level": "info"}"#); // Other valid field
    assert!(load_config_from_file(p.to_str().unwrap()).is_ok());
    fs::remove_file(p).ok();

    // enabled = true
    init();
    let p = temp_json_file(r#"{"enabled": "true"}"#);
    assert!(load_config_from_file(p.to_str().unwrap()).is_ok());
    fs::remove_file(p).ok();

    // enabled=invalid → ParseError("incorrect value given")
    init();
    let p = temp_json_file(r#"{"enabled": "maybe"}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(
        matches!(&res, Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"),
        "Unexpected error: {:?}",
        res
    );
    fs::remove_file(p).ok();
}

#[test]
fn json_all_levels_valid() {
    let cases = [
        ("trace", Level::TRACE),
        ("debug", Level::DEBUG),
        ("info", Level::INFO),
        ("warn", Level::WARN),
        ("error", Level::ERROR),
        ("TRACE", Level::TRACE), // Test uppercase
    ];

    for (lvl_str, lvl_enum) in cases {
        init();
        let content = format!(r#"{{"level": "{}"}}"#, lvl_str);
        let p = temp_json_file(&content);
        assert!(load_config_from_file(p.to_str().unwrap()).is_ok());
        let cfg = config_snapshot();
        assert_eq!(cfg.level, lvl_enum, "level `{}` not applied", lvl_str);
        fs::remove_file(p).ok();
    }

    // invalid level value
    init();
    let p = temp_json_file(r#"{"level": "verbose"}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(
        matches!(&res, Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"),
        "Unexpected error: {:?}",
        res
    );
    fs::remove_file(p).ok();
}

#[test]
fn json_print_to_terminal_variants() {
    init(); // true
    let p_true = temp_json_file(r#"{"print_to_terminal": "true"}"#);
    assert!(load_config_from_file(p_true.to_str().unwrap()).is_ok());
    assert!(config_snapshot().print_to_terminal);
    fs::remove_file(p_true).ok();

    init(); // false
    let p_false = temp_json_file(r#"{"print_to_terminal": "false"}"#);
    assert!(load_config_from_file(p_false.to_str().unwrap()).is_ok());
    assert!(!config_snapshot().print_to_terminal);
    fs::remove_file(p_false).ok();

    init(); // invalid
    let p_invalid = temp_json_file(r#"{"print_to_terminal": "nope"}"#);
    let res = load_config_from_file(p_invalid.to_str().unwrap());
    assert!(
        matches!(&res, Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"),
        "Unexpected error: {:?}",
        res
    );
    fs::remove_file(p_invalid).ok();
}

#[test]
fn json_colorized_variants() {
    init(); // true
    let p_true = temp_json_file(r#"{"colorized": "true"}"#);
    assert!(load_config_from_file(p_true.to_str().unwrap()).is_ok());
    assert!(config_snapshot().colorized);
    fs::remove_file(p_true).ok();

    init(); // false
    let p_false = temp_json_file(r#"{"colorized": "false"}"#);
    assert!(load_config_from_file(p_false.to_str().unwrap()).is_ok());
    assert!(!config_snapshot().colorized);
    fs::remove_file(p_false).ok();

    init(); // invalid
    let p_invalid = temp_json_file(r#"{"colorized": "rainbow"}"#);
    let res = load_config_from_file(p_invalid.to_str().unwrap());
    assert!(
        matches!(&res, Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"),
        "Unexpected error: {:?}",
        res
    );
    fs::remove_file(p_invalid).ok();
}

#[test]
fn json_global_formatting_ok_and_bad() {
    init(); // Valid
    let fmt_txt = "[{level}] {message}";
    let p_ok = temp_json_file(&format!(r#"{{"global_formatting": "{}"}}"#, fmt_txt));
    assert!(load_config_from_file(p_ok.to_str().unwrap()).is_ok());

    let cfg = config_snapshot();
    let expected = LogFormatter::parse_from_string(fmt_txt).unwrap();
    assert_eq!(cfg.trace_log_format.parts, expected.parts);
    assert_eq!(cfg.debug_log_format.parts, expected.parts);
    assert_eq!(cfg.info_log_format.parts, expected.parts);
    assert_eq!(cfg.warn_log_format.parts, expected.parts);
    assert_eq!(cfg.error_log_format.parts, expected.parts);
    fs::remove_file(p_ok).ok();

    init(); // Invalid
    let bad_fmt = "<red>[{level}]"; // missing closing <red>
    let p_bad = temp_json_file(&format!(r#"{{"global_formatting": "{}"}}"#, bad_fmt));
    let res = load_config_from_file(p_bad.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetLevelFormatting(
            SetLevelFormattingError::IncorrectFormatGiven(_)
        ))
    ));
    fs::remove_file(p_bad).ok();
}

#[test]
fn json_individual_level_formatting() {
    init();
    let trace_fmt_txt = "TRACE: {message}";
    let error_fmt_txt = "ERROR: {file}:{line} {message}";
    let content = format!(
        r#"{{
        "trace_formatting": "{}",
        "error_formatting": "{}"
    }}"#,
        trace_fmt_txt, error_fmt_txt
    );
    let p = temp_json_file(&content);
    assert!(load_config_from_file(p.to_str().unwrap()).is_ok());

    let cfg = config_snapshot();
    let default_fmt = LogFormatter::default(); // Other levels should remain default
    assert_eq!(
        cfg.trace_log_format.parts,
        LogFormatter::parse_from_string(trace_fmt_txt)
            .unwrap()
            .parts
    );
    assert_eq!(cfg.debug_log_format.parts, default_fmt.parts); // Assuming default is not changed
    assert_eq!(
        cfg.error_log_format.parts,
        LogFormatter::parse_from_string(error_fmt_txt)
            .unwrap()
            .parts
    );
    fs::remove_file(p).ok();

    // Test invalid individual format
    init();
    let p_bad = temp_json_file(r#"{"info_formatting": "<blue>{message}"}"#); // missing closing </blue>
    let res = load_config_from_file(p_bad.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetLevelFormatting(
            SetLevelFormattingError::IncorrectFormatGiven(_)
        ))
    ));
    fs::remove_file(p_bad).ok();
}

#[test]
fn json_full_file_config_valid() {
    init();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let archive_dir_name = format!("test_json_archives_full_{}", ts);

    let content = format!(
        r#"{{
        "file_name": "app_{{date}}_{{time}}.log",
        "compression": "zip",
        "rotations": ["1 day", "10 MB", "12:30"],
        "archive_dir": "{}"
    }}"#,
        archive_dir_name
    );
    let p = temp_json_file(&content);
    let load_res = load_config_from_file(p.to_str().unwrap());
    assert!(
        load_res.is_ok(),
        "load_config_from_file failed: {:?}",
        load_res.err()
    );

    let cfg = config_snapshot();
    assert!(
        cfg.file_manager.is_some(),
        "File manager should be configured"
    );
    assert_eq!(cfg.archive_dir, Some(PathBuf::from(&archive_dir_name)));
    assert!(
        Path::new(&archive_dir_name).is_dir(),
        "Archive directory was not created"
    );

    // Check rotations and compression via debug output of file_manager
    let fm_lock = cfg.file_manager.as_ref().unwrap().lock().unwrap();
    let fm_dbg = format!("{:?}", fm_lock);
    println!("{:?}", fm_dbg);
    assert!(fm_dbg.contains("Zip"));
    assert!(fm_dbg.contains("Period")); // "1 day"
    assert!(fm_dbg.contains("Size")); // "10 MB"
    assert!(fm_dbg.contains("Time")); // "12:30"

    cleanup_archive_dir(&archive_dir_name);
    fs::remove_file(p).ok();
}

#[test]
fn json_file_name_invalid_format() {
    init();
    // Forbidden character '<'
    let p = temp_json_file(r#"{"file_name": "test<bad>.log"}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetFile(
            SetFileError::UnableToLoadFromString(_)
        ))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn json_compression_without_file_name() {
    init();
    let p = temp_json_file(r#"{"compression": "zip"}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetCompression(
            SetCompressionError::FileIsntSet
        ))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn json_invalid_compression_value() {
    init();
    let p = temp_json_file(r#"{"file_name": "app.log", "compression": "rar"}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetCompression(
            SetCompressionError::IncorrectCompressionValue
        ))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn json_rotations_without_file_name() {
    init();
    let p = temp_json_file(r#"{"rotations": ["1 day"]}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::AddRotation(
            AddRotationError::FileIsntSet
        ))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn json_invalid_rotation_value() {
    init();
    let p = temp_json_file(r#"{"file_name": "app.log", "rotations": ["invalid rotation"]}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::AddRotation(
            AddRotationError::IncorrectFormatGiven
        ))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn json_empty_rotations_array() {
    init();
    let p = temp_json_file(r#"{"file_name": "app.log", "rotations": []}"#);
    let load_res = load_config_from_file(p.to_str().unwrap());
    assert!(
        load_res.is_ok(),
        "load_config_from_file failed for empty rotations: {:?}",
        load_res.err()
    );

    let cfg = config_snapshot();
    let fm_lock = cfg.file_manager.as_ref().unwrap().lock().unwrap();
    let fm_dbg = format!("{:?}", fm_lock);
    // Check that rotation list is empty in debug string
    assert!(fm_dbg.contains("rotation: []"));
    fs::remove_file(p).ok();
}

#[test]
fn json_archive_dir_creates_directory() {
    init();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let archive_dir_name = format!("test_json_archive_dir_creation_{}", ts);

    let content = format!(r#"{{"archive_dir": "{}"}}"#, archive_dir_name);
    let p = temp_json_file(&content);
    let result = load_config_from_file(p.to_str().unwrap());
    assert!(
        result.is_ok(),
        "load_config_from_file failed: {:?}",
        result.err()
    );

    let cfg = config_snapshot();
    assert_eq!(cfg.archive_dir, Some(PathBuf::from(&archive_dir_name)));
    assert!(
        Path::new(&archive_dir_name).is_dir(),
        "Archive directory was not created"
    );

    cleanup_archive_dir(&archive_dir_name);
    fs::remove_file(p).ok();
}

#[test]
fn json_archive_dir_path_is_file() {
    init();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_acting_as_dir_name = format!("test_json_file_as_dir_{}", ts);

    File::create(&file_acting_as_dir_name)
        .unwrap()
        .write_all(b"i am a file")
        .unwrap();

    let content = format!(r#"{{"archive_dir": "{}"}}"#, file_acting_as_dir_name);
    let p = temp_json_file(&content);
    let result = load_config_from_file(p.to_str().unwrap());

    assert!(matches!(
        result,
        Err(ReadFromConfigFileError::SetArchiveDirError(
            SetArchiveDirError::UnableToCreateDir(_)
        ))
    ));

    fs::remove_file(file_acting_as_dir_name).ok();
    fs::remove_file(p).ok();
}

#[test]
fn json_empty_file() {
    init();
    let p = temp_json_file(""); // Empty content
    let res = load_config_from_file(p.to_str().unwrap());
    // serde_json::from_str on empty string gives "EOF while parsing a value"
    assert!(matches!(&res, Err(ReadFromConfigFileError::ParseError(s)) if s.contains("EOF")));
    fs::remove_file(p).ok();
}

#[test]
fn json_malformed_file() {
    init();
    let p = temp_json_file(r#"{"level": "info",malformed}"#); // Malformed JSON
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(&res, Err(ReadFromConfigFileError::ParseError(_))));
    fs::remove_file(p).ok();
}

#[test]
fn json_unknown_fields_ignored() {
    init();
    let p = temp_json_file(r#"{"level": "warn", "unknown_field": "some_value"}"#);
    let load_res = load_config_from_file(p.to_str().unwrap());
    assert!(
        load_res.is_ok(),
        "load_config_from_file failed: {:?}",
        load_res.err()
    );
    // Check that known field was applied
    assert_eq!(config_snapshot().level, Level::WARN);
    fs::remove_file(p).ok();
}

#[test]
fn json_incorrect_value_types() {
    init();
    // level expects a string like "info", not a number
    let p = temp_json_file(r#"{"level": 123}"#);
    let res = load_config_from_file(p.to_str().unwrap());
    // serde_json will fail to deserialize: invalid type: integer `123`, expected a string
    assert!(
        matches!(&res, Err(ReadFromConfigFileError::ParseError(s)) if s.contains("invalid type"))
    );
    fs::remove_file(p).ok();

    init();
    // rotations expects an array of strings, not a single string
    let p_rot_str = temp_json_file(r#"{"rotations": "1 day"}"#);
    let res_rot_str = load_config_from_file(p_rot_str.to_str().unwrap());
    assert!(
        matches!(&res_rot_str, Err(ReadFromConfigFileError::ParseError(s)) if s.contains("invalid type"))
    );
    fs::remove_file(p_rot_str).ok();
}

#[test]
fn json_missing_file() {
    init();
    let bogus_path = "/no/such/path/to_json_file.json";
    let res = load_config_from_file(bogus_path);
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::ReadFileError(_))
    ));
}

#[test]
fn json_incorrect_extension_but_valid_json_content() {
    init();
    // File has .txt extension but JSON content, load_config_from_file should dispatch by extension
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("loggit_json_test_wrong_ext_{}.txt", ts)); // Wrong extension
    let mut file = File::create(&path).unwrap();
    write!(file, r#"{{"level": "error"}}"#).unwrap(); // Valid JSON content

    let res = load_config_from_file(path.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::IncorrectFileExtension)
    ));
    fs::remove_file(path).ok();
}

#[test]
fn json_file_name_without_extension_in_path() {
    init();
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("loggit_json_test_no_ext_{}", ts)); // No extension in path

    // We don't need to create the file, as the error should occur before reading.
    // `path.to_str().unwrap()` is the argument to `load_config_from_file`.
    // `load_config_from_file` calls `parse_config_file` which checks `path.contains(".")`

    let res = load_config_from_file(path.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::IncorrectFileName)
    ));
    // No file to remove as it wasn't necessarily created by helper if path itself is bad
}
