use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use once_cell::sync::Lazy;

use crate::logger::formatter::LogFormatter;
use crate::Config as LoggerConfig;
use crate::{
    logger::{
        self, // for init, info!, etc.
        from_env::load_config_from_env,
        set_errors::{
            AddRotationError, ReadFromConfigFileError, SetArchiveDirError, SetColorizedError,
            SetCompressionError, SetFileError, SetLevelFormattingError,
        },
    },
    Level, CONFIG,
};

// RAII guard for managing a single environment variable
struct EnvVarGuard {
    key: String,
    original_value: Option<String>,
}

impl EnvVarGuard {
    fn new(key: &str, value: &str) -> Self {
        let key_string = key.to_string();
        let original_value = env::var(&key_string).ok();
        env::set_var(key, value);
        EnvVarGuard {
            key: key_string,
            original_value,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(original_value) = &self.original_value {
            env::set_var(&self.key, original_value);
        } else {
            env::remove_var(&self.key);
        }
    }
}

// RAII guard for managing multiple environment variables, useful for clearing
struct EnvVarsGuard {
    vars: Vec<(String, Option<String>)>, // key, original_value
}

impl EnvVarsGuard {
    fn clear(vars_to_clear: &[&str]) -> Self {
        let mut original_vars = Vec::new();
        for key in vars_to_clear {
            original_vars.push((key.to_string(), env::var(*key).ok()));
            env::remove_var(*key);
        }
        EnvVarsGuard {
            vars: original_vars,
        }
    }
}

impl Drop for EnvVarsGuard {
    fn drop(&mut self) {
        for (key, original_value) in &self.vars {
            if let Some(val) = original_value {
                env::set_var(key, val);
            } else {
                env::remove_var(key);
            }
        }
    }
}

fn config_snapshot() -> LoggerConfig {
    CONFIG.read().expect("CONFIG should be readable").clone()
}

fn cleanup_archive_dir(path_str: &str) {
    let path = Path::new(path_str);
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(path).ok();
        println!("Cleaned up archive dir: {}", path_str);
    }
}

fn cleanup_log_files(prefix: &str) {
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.filter_map(Result::ok) {
            let p = entry.path();
            if p.is_file() {
                if let Some(name_str) = p.file_name().and_then(|n| n.to_str()) {
                    if name_str.starts_with(prefix)
                        && (name_str.ends_with(".log") || name_str.ends_with(".txt"))
                    {
                        fs::remove_file(&p).ok();
                        println!("Cleaned up log file: {:?}", p);
                    }
                }
            }
        }
    }
}

const ALL_CONFIG_KEYS: &[&str] = &[
    "level",
    "print_to_terminal",
    "colorized",
    "global_formatting",
    "trace_formatting",
    "debug_formatting",
    "info_formatting",
    "warn_formatting",
    "error_formatting",
    "file_name",
    "compression",
    "rotations",
    "archive_dir",
];

#[test]
fn env_no_vars_set() {
    logger::init();
    let initial_config = config_snapshot();

    let _clear_guard = EnvVarsGuard::clear(ALL_CONFIG_KEYS);

    let result = load_config_from_env();
    assert!(result.is_ok());

    // Config should remain unchanged (default)
    assert_eq!(config_snapshot().level, initial_config.level);
    assert_eq!(
        config_snapshot().print_to_terminal,
        initial_config.print_to_terminal
    );
    // ... and so on for all fields if we want to be extremely thorough,
    // or just trust that no setters were called.
}

#[test]
fn env_enabled_var_has_no_effect() {
    // The `enabled` field is not read by `parse_config_from_env`.
    // So setting it via an env var should not disable the logger or cause DisabledToBeUsed error.
    logger::init();
    let original_config = config_snapshot(); // Should be enabled by default

    // "enabled" is not in ALL_CONFIG_KEYS because the loader doesn't look for it.
    // We set it to show it has no effect.
    let _guard = EnvVarGuard::new("enabled", "false");

    let result = load_config_from_env();
    assert!(result.is_ok(), "load_config_from_env should succeed");
    assert_eq!(
        config_snapshot().print_to_terminal, // Check a field to see if config is default
        original_config.print_to_terminal,
        "Config should not have changed due to 'enabled' env var"
    );
    // The logger remains enabled (its default state or previous state is not affected by this non-read env var).
}

#[test]
fn env_level_variants() {
    let levels_to_test = [
        ("trace", Level::TRACE),
        ("DEBUG", Level::DEBUG), // check case-insensitivity (as per parser)
        ("info", Level::INFO),
        ("WaRn", Level::WARN),
        ("error", Level::ERROR),
    ];

    for (level_str, expected_level) in levels_to_test {
        logger::init();
        let _guard = EnvVarGuard::new("level", level_str);
        let result = load_config_from_env();
        assert!(result.is_ok());
        assert_eq!(config_snapshot().level, expected_level);
    }

    // Invalid level
    logger::init();
    let _guard_invalid = EnvVarGuard::new("level", "verbose");
    let result_invalid = load_config_from_env();
    assert!(matches!(
        result_invalid,
        Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"
    ));

    // Empty level
    logger::init();
    let _guard_empty = EnvVarGuard::new("level", "");
    let result_empty = load_config_from_env();
    assert!(matches!(
        result_empty,
        Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"
    ));
}

#[test]
fn env_print_to_terminal_variants() {
    // true
    logger::init();
    let _guard_true = EnvVarGuard::new("print_to_terminal", "true");
    assert!(load_config_from_env().is_ok());
    assert!(config_snapshot().print_to_terminal);

    // false
    logger::init();
    let _guard_false = EnvVarGuard::new("print_to_terminal", "false");
    assert!(load_config_from_env().is_ok());
    assert!(!config_snapshot().print_to_terminal);

    // invalid
    logger::init();
    let _guard_invalid = EnvVarGuard::new("print_to_terminal", "maybe");
    let result_invalid = load_config_from_env();
    assert!(matches!(
        result_invalid,
        Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"
    ));

    // empty
    logger::init();
    let _guard_empty = EnvVarGuard::new("print_to_terminal", "");
    let result_empty = load_config_from_env();
    assert!(matches!(
        result_empty,
        Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"
    ));
}

#[test]
fn env_colorized_variants() {
    // true
    logger::init();
    let _guard_true = EnvVarGuard::new("colorized", "true");
    assert!(load_config_from_env().is_ok());
    assert!(config_snapshot().colorized);

    // false
    logger::init();
    let _guard_false = EnvVarGuard::new("colorized", "false");
    assert!(load_config_from_env().is_ok());
    assert!(!config_snapshot().colorized);

    // invalid
    logger::init();
    let _guard_invalid = EnvVarGuard::new("colorized", "rainbow");
    let result_invalid = load_config_from_env();
    assert!(matches!(
        result_invalid,
        Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"
    ));

    // empty
    logger::init();
    let _guard_empty = EnvVarGuard::new("colorized", "");
    let result_empty = load_config_from_env();
    assert!(matches!(
        result_empty,
        Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"
    ));
}

#[test]
fn env_global_formatting() {
    // Valid
    logger::init();
    let fmt_valid = "{level} - {message}";
    let _guard_valid = EnvVarGuard::new("global_formatting", fmt_valid);
    assert!(load_config_from_env().is_ok());
    let cfg = config_snapshot();
    let expected_fmt = LogFormatter::parse_from_string(fmt_valid).unwrap();
    assert_eq!(cfg.trace_log_format.parts, expected_fmt.parts);
    assert_eq!(cfg.info_log_format.parts, expected_fmt.parts); // Check one more

    // Invalid (unclosed tag)
    logger::init();
    let fmt_invalid = "<red>{level} - {message}";
    let _guard_invalid = EnvVarGuard::new("global_formatting", fmt_invalid);
    let result_invalid = load_config_from_env();
    assert!(matches!(
        result_invalid,
        Err(ReadFromConfigFileError::SetLevelFormatting(
            SetLevelFormattingError::IncorrectFormatGiven(_)
        ))
    ));

    // Empty (should be valid, resulting in an empty formatter)
    logger::init();
    let _guard_empty = EnvVarGuard::new("global_formatting", "");
    assert!(load_config_from_env().is_ok());
    let cfg_empty = config_snapshot();
    assert!(cfg_empty.info_log_format.parts.is_empty());
}

#[test]
fn env_specific_level_formatting() {
    logger::init();
    let default_config = config_snapshot(); // To compare other levels against

    let info_fmt_str = "INFO-SPECIFIC: {message}";
    let _guard = EnvVarGuard::new("info_formatting", info_fmt_str);
    assert!(load_config_from_env().is_ok());

    let cfg = config_snapshot();
    let expected_info_fmt = LogFormatter::parse_from_string(info_fmt_str).unwrap();
    assert_eq!(cfg.info_log_format.parts, expected_info_fmt.parts);
    // Check that other levels retain their default formatting
    assert_eq!(
        cfg.trace_log_format.parts,
        default_config.trace_log_format.parts
    );
    assert_eq!(
        cfg.debug_log_format.parts,
        default_config.debug_log_format.parts
    );

    // Invalid specific format
    logger::init();
    let _guard_invalid = EnvVarGuard::new("warn_formatting", "<yellow>{message}"); // Unclosed
    let result_invalid = load_config_from_env();
    assert!(matches!(
        result_invalid,
        Err(ReadFromConfigFileError::SetLevelFormatting(
            SetLevelFormattingError::IncorrectFormatGiven(_)
        ))
    ));
}

#[test]
fn env_global_and_specific_formatting_interaction() {
    logger::init();

    let global_fmt_str = "GLOBAL: {level} {message}";
    let info_fmt_str = "INFO-SPECIFIC: {message}";

    // Need multiple guards; simplest to recreate them or use a Vec.
    // For this test, creating them one by one is fine as they are independent for setup.
    let _guard_global = EnvVarGuard::new("global_formatting", global_fmt_str);
    let _guard_info = EnvVarGuard::new("info_formatting", info_fmt_str);

    assert!(load_config_from_env().is_ok());

    let cfg = config_snapshot();
    let expected_global_fmt = LogFormatter::parse_from_string(global_fmt_str).unwrap();
    let expected_info_fmt = LogFormatter::parse_from_string(info_fmt_str).unwrap();

    assert_eq!(cfg.trace_log_format.parts, expected_global_fmt.parts);
    assert_eq!(cfg.debug_log_format.parts, expected_global_fmt.parts);
    assert_eq!(cfg.info_log_format.parts, expected_info_fmt.parts); // Specific overrides global
    assert_eq!(cfg.warn_log_format.parts, expected_global_fmt.parts);
    assert_eq!(cfg.error_log_format.parts, expected_global_fmt.parts);
}

#[test]
fn env_file_config() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_pattern = format!("env_test_file_{}_{{date}}.log", ts);
    let file_prefix = format!("env_test_file_{}", ts);

    // Valid
    logger::init();
    // Ensure terminal output is off or tests might be noisy. Or check file content only.
    logger::set_print_to_terminal(false).unwrap();

    let _guard_valid = EnvVarGuard::new("file_name", &file_pattern);
    assert!(load_config_from_env().is_ok());
    let cfg = config_snapshot();
    assert!(cfg.file_manager.is_some());

    // Log something to trigger file creation
    crate::info!("Test message for env_file_config");
    std::thread::sleep(std::time::Duration::from_millis(100)); // Give time for flush

    let mut log_file_exists = false;
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.filter_map(Result::ok) {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with(&file_prefix)
            {
                log_file_exists = true;
                break;
            }
        }
    }
    assert!(
        log_file_exists,
        "Log file was not created for pattern: {}",
        file_pattern
    );
    cleanup_log_files(&file_prefix);

    // Invalid (bad char)
    logger::init();
    let _guard_invalid = EnvVarGuard::new("file_name", "test<bad>.log");
    let result_invalid = load_config_from_env();
    assert!(matches!(
        result_invalid,
        Err(ReadFromConfigFileError::SetFile(
            SetFileError::UnableToLoadFromString(_)
        ))
    ));
    cleanup_log_files("test<bad>"); // just in case

    // Empty
    logger::init();
    let _guard_empty = EnvVarGuard::new("file_name", "");
    let result_empty = load_config_from_env();
    assert!(matches!(
        result_empty,
        Err(ReadFromConfigFileError::SetFile(
            SetFileError::UnableToLoadFromString(_)
        )) // EmptyStringGiven
    ));
}

#[test]
fn env_compression_config() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_pattern = format!("env_comp_test_{}.log", ts); // Simpler pattern for this test

    // Valid (with file set first)
    logger::init();
    let _guard_file = EnvVarGuard::new("file_name", &file_pattern);
    let _guard_comp = EnvVarGuard::new("compression", "zip");
    assert!(load_config_from_env().is_ok());
    let cfg = config_snapshot();
    assert!(cfg.file_manager.is_some());
    // Check internal state if possible, or rely on behavior (e.g. rotation creates zip)
    // For now, just check it was accepted:
    let fm_dbg = format!("{:?}", cfg.file_manager.as_ref().unwrap().lock().unwrap());
    assert!(fm_dbg.contains("Zip"));
    cleanup_log_files(&file_pattern[..file_pattern.rfind('.').unwrap_or(file_pattern.len())]);

    // Invalid compression type
    logger::init();
    let _guard_file_2 = EnvVarGuard::new("file_name", &file_pattern);
    let _guard_comp_invalid = EnvVarGuard::new("compression", "rar");
    let result_invalid = load_config_from_env();
    assert!(matches!(
        result_invalid,
        Err(ReadFromConfigFileError::SetCompression(
            SetCompressionError::IncorrectCompressionValue
        ))
    ));
    cleanup_log_files(&file_pattern[..file_pattern.rfind('.').unwrap_or(file_pattern.len())]);

    // Compression without file
    logger::init();
    // No "file_name" guard here
    //let _guard_comp_nofile = EnvVarGuard::new("compression", "zip");
    //let result_nofile = load_config_from_env();
    //assert!(matches!(
    //    result_nofile,
    //    Err(ReadFromConfigFileError::SetCompression(
    //        SetCompressionError::FileIsntSet
    //    ))
    //));

    // Empty compression type
    logger::init();
    let _guard_file_3 = EnvVarGuard::new("file_name", &file_pattern);
    let _guard_comp_empty = EnvVarGuard::new("compression", "");
    let result_empty = load_config_from_env();
    assert!(matches!(
        result_empty,
        Err(ReadFromConfigFileError::SetCompression(
            SetCompressionError::IncorrectCompressionValue
        ))
    ));
    cleanup_log_files(&file_pattern[..file_pattern.rfind('.').unwrap_or(file_pattern.len())]);
}

#[test]
fn env_rotations_config() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_pattern = format!("env_rot_test_{}.log", ts);

    // Valid single rotation
    logger::init();
    let _g_file1 = EnvVarGuard::new("file_name", &file_pattern);
    let _g_rot1 = EnvVarGuard::new("rotations", "1 day");
    assert!(load_config_from_env().is_ok());
    let fm_dbg1 = format!(
        "{:?}",
        config_snapshot().file_manager.unwrap().lock().unwrap()
    );
    assert!(fm_dbg1.matches("Rotation { rotation_type: Period").count() == 1);
    cleanup_log_files(&file_pattern[..file_pattern.rfind('.').unwrap_or(file_pattern.len())]);

    // Valid multiple rotations, comma-separated with spaces
    logger::init();
    let _g_file2 = EnvVarGuard::new("file_name", &file_pattern);
    let _g_rot2 = EnvVarGuard::new("rotations", "10 MB,  12:30 ");
    assert!(load_config_from_env().is_ok());
    let fm_dbg2 = format!(
        "{:?}",
        config_snapshot().file_manager.unwrap().lock().unwrap()
    );
    assert!(fm_dbg2.matches("Rotation { rotation_type:").count() == 2);
    assert!(fm_dbg2.contains("Size"));
    assert!(fm_dbg2.contains("Time"));
    cleanup_log_files(&file_pattern[..file_pattern.rfind('.').unwrap_or(file_pattern.len())]);

    // Invalid rotation
    logger::init();
    let _g_file3 = EnvVarGuard::new("file_name", &file_pattern);
    let _g_rot_invalid = EnvVarGuard::new("rotations", "bad value");
    let res_invalid = load_config_from_env();
    assert!(matches!(
        res_invalid,
        Err(ReadFromConfigFileError::AddRotation(
            AddRotationError::IncorrectFormatGiven
        ))
    ));
    cleanup_log_files(&file_pattern[..file_pattern.rfind('.').unwrap_or(file_pattern.len())]);

    // Rotations without file
    //logger::init();
    //let _g_rot_nofile = EnvVarGuard::new("rotations", "1 day");
    //let res_nofile = load_config_from_env();
    //assert!(matches!(
    //    res_nofile,
    //    Err(ReadFromConfigFileError::AddRotation(
    //        AddRotationError::FileIsntSet
    //    ))
    //));

    // Empty rotation string
    logger::init();
    let _g_file4 = EnvVarGuard::new("file_name", &file_pattern);
    let _g_rot_empty = EnvVarGuard::new("rotations", "");
    let res_empty = load_config_from_env();
    assert!(matches!(
        res_empty,
        Err(ReadFromConfigFileError::AddRotation(
            AddRotationError::IncorrectFormatGiven
        ))
    ));
    cleanup_log_files(&file_pattern[..file_pattern.rfind('.').unwrap_or(file_pattern.len())]);

    // Comma with empty parts
    logger::init();
    let _g_file5 = EnvVarGuard::new("file_name", &file_pattern);
    let _g_rot_comma = EnvVarGuard::new("rotations", ",1 day,"); // first part empty
    let res_comma = load_config_from_env();
    assert!(matches!(
        res_comma,
        Err(ReadFromConfigFileError::AddRotation(
            AddRotationError::IncorrectFormatGiven // Fails on the first empty part
        ))
    ));
    cleanup_log_files(&file_pattern[..file_pattern.rfind('.').unwrap_or(file_pattern.len())]);
}

#[test]
fn env_archive_dir_config() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let archive_dir_name = format!("env_test_archives_{}", ts);

    // Valid
    logger::init();
    let _guard_valid = EnvVarGuard::new("archive_dir", &archive_dir_name);
    assert!(load_config_from_env().is_ok());
    let cfg = config_snapshot();
    assert_eq!(cfg.archive_dir, Some(PathBuf::from(&archive_dir_name)));
    assert!(Path::new(&archive_dir_name).is_dir());
    cleanup_archive_dir(&archive_dir_name);

    // Path is a file
    logger::init();
    let file_as_dir_name = format!("env_test_file_as_dir_{}", ts);
    fs::File::create(&file_as_dir_name)
        .unwrap()
        .write_all(b"content")
        .unwrap();
    assert!(Path::new(&file_as_dir_name).is_file());

    let _guard_file_path = EnvVarGuard::new("archive_dir", &file_as_dir_name);
    let result_file_path = load_config_from_env();
    assert!(matches!(
        result_file_path,
        Err(ReadFromConfigFileError::SetArchiveDirError(
            SetArchiveDirError::UnableToCreateDir(_)
        ))
    ));
    fs::remove_file(&file_as_dir_name).ok();
    cleanup_archive_dir(&file_as_dir_name); // Attempt cleanup in case it became a dir
}

#[test]
fn env_partial_config() {
    logger::init();
    let default_config = config_snapshot();

    let _guard_level = EnvVarGuard::new("level", "error");
    let _guard_color = EnvVarGuard::new("colorized", "true");

    // Clear other potential vars that might interfere if tests run in weird sequence without full isolation
    let vars_to_ensure_unset: Vec<&str> = ALL_CONFIG_KEYS
        .iter()
        .filter(|&&k| k != "level" && k != "colorized")
        .cloned()
        .collect();
    let _clear_others = EnvVarsGuard::clear(&vars_to_ensure_unset);

    assert!(load_config_from_env().is_ok());

    let cfg = config_snapshot();
    assert_eq!(cfg.level, Level::ERROR);
    assert!(cfg.colorized);
    // Check a field that wasn't set, it should be default
    assert_eq!(cfg.print_to_terminal, default_config.print_to_terminal);
    assert_eq!(
        cfg.info_log_format.parts,
        default_config.info_log_format.parts
    );
}
