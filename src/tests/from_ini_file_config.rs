// ./src/tests/from_ini_file_config.rs
#![allow(unused_imports)] // temp for dev
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::logger::formatter::LogFormatter;
use crate::Config as LoggerConfig;
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

// Helper to create a temporary INI file
fn temp_ini_file(contents: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_name = format!("loggit_ini_test_{}.ini", ts);
    path.push(file_name);

    let mut file = File::create(&path)
        .unwrap_or_else(|e| panic!("Failed to create temp ini-file at {:?}: {}", path, e));
    write!(file, "{}", contents)
        .unwrap_or_else(|e| panic!("Failed to write temp ini-file at {:?}: {}", path, e));
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
fn ini_enabled_variants() {
    init(); // enabled=false
    let p_false = temp_ini_file("[Config]\nenabled=false\n");
    let res_false = load_config_from_file(p_false.to_str().unwrap());
    assert!(matches!(
        res_false,
        Err(ReadFromConfigFileError::DisabledToBeUsed)
    ));
    fs::remove_file(p_false).ok();

    init(); // enabled missing
    let p_missing = temp_ini_file("[Config]\nlevel=info\n");
    assert!(load_config_from_file(p_missing.to_str().unwrap()).is_ok());
    fs::remove_file(p_missing).ok();

    init(); // enabled=true
    let p_true = temp_ini_file("[Config]\nenabled=true\n");
    assert!(load_config_from_file(p_true.to_str().unwrap()).is_ok());
    fs::remove_file(p_true).ok();

    init(); // enabled=invalid
    let p_invalid = temp_ini_file("[Config]\nenabled=maybe\n");
    let res_invalid = load_config_from_file(p_invalid.to_str().unwrap());
    assert!(
        matches!(&res_invalid, Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"),
        "Unexpected error: {:?}",
        res_invalid
    );
    fs::remove_file(p_invalid).ok();
}

#[test]
fn ini_all_levels_valid() {
    let cases = [
        ("trace", Level::TRACE),
        ("debug", Level::DEBUG),
        ("info", Level::INFO),
        ("warn", Level::WARN),
        ("error", Level::ERROR),
        ("ERROR", Level::ERROR),
    ];
    for (lvl_str, lvl_enum) in cases {
        init();
        let content = format!("[Config]\nlevel={}\n", lvl_str);
        let p = temp_ini_file(&content);
        assert!(load_config_from_file(p.to_str().unwrap()).is_ok());
        assert_eq!(
            config_snapshot().level,
            lvl_enum,
            "Level '{}' not applied",
            lvl_str
        );
        fs::remove_file(p).ok();
    }

    init(); // invalid level
    let p_invalid = temp_ini_file("[Config]\nlevel=verbose\n");
    let res_invalid = load_config_from_file(p_invalid.to_str().unwrap());
    assert!(
        matches!(&res_invalid, Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"),
        "Unexpected error: {:?}",
        res_invalid
    );
    fs::remove_file(p_invalid).ok();
}

#[test]
fn ini_print_to_terminal_variants() {
    init(); // true
    let p_true = temp_ini_file("[Config]\nprint_to_terminal=true\n");
    assert!(load_config_from_file(p_true.to_str().unwrap()).is_ok());
    assert!(config_snapshot().print_to_terminal);
    fs::remove_file(p_true).ok();

    init(); // false
    let p_false = temp_ini_file("[Config]\nprint_to_terminal=false\n");
    assert!(load_config_from_file(p_false.to_str().unwrap()).is_ok());
    assert!(!config_snapshot().print_to_terminal);
    fs::remove_file(p_false).ok();

    init(); // invalid
    let p_invalid = temp_ini_file("[Config]\nprint_to_terminal=nope\n");
    let res_invalid = load_config_from_file(p_invalid.to_str().unwrap());
    assert!(
        matches!(&res_invalid, Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"),
        "Unexpected error: {:?}",
        res_invalid
    );
    fs::remove_file(p_invalid).ok();
}

#[test]
fn ini_colorized_variants() {
    init(); // true
    let p_true = temp_ini_file("[Config]\ncolorized=true\n");
    assert!(load_config_from_file(p_true.to_str().unwrap()).is_ok());
    assert!(config_snapshot().colorized);
    fs::remove_file(p_true).ok();

    init(); // false
    let p_false = temp_ini_file("[Config]\ncolorized=false\n");
    assert!(load_config_from_file(p_false.to_str().unwrap()).is_ok());
    assert!(!config_snapshot().colorized);
    fs::remove_file(p_false).ok();

    init(); // invalid
    let p_invalid = temp_ini_file("[Config]\ncolorized=rainbow\n");
    let res_invalid = load_config_from_file(p_invalid.to_str().unwrap());
    assert!(
        matches!(&res_invalid, Err(ReadFromConfigFileError::ParseError(s)) if s == "incorrect value given"),
        "Unexpected error: {:?}",
        res_invalid
    );
    fs::remove_file(p_invalid).ok();
}

#[test]
fn ini_global_formatting_ok_and_bad() {
    init(); // Valid
    let fmt_txt = "[{level}] {message}";
    let p_ok = temp_ini_file(&format!("[Config]\nglobal_formatting={}\n", fmt_txt));
    assert!(load_config_from_file(p_ok.to_str().unwrap()).is_ok());
    let cfg = config_snapshot();
    let expected = LogFormatter::parse_from_string(fmt_txt).unwrap();
    assert_eq!(cfg.trace_log_format.parts, expected.parts);
    // ... (check other levels)
    fs::remove_file(p_ok).ok();

    init(); // Invalid
    let bad_fmt = "<red>[{level}]"; // missing closing <red>
    let p_bad = temp_ini_file(&format!("[Config]\nglobal_formatting={}\n", bad_fmt));
    let res = load_config_from_file(p_bad.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetLevelFormatting(_))
    ));
    fs::remove_file(p_bad).ok();
}

#[test]
fn ini_individual_level_formatting() {
    init();
    let info_fmt_txt = "INFO: {message}";
    let content = format!("[Config]\ninfo_formatting={}\n", info_fmt_txt);
    let p = temp_ini_file(&content);
    assert!(load_config_from_file(p.to_str().unwrap()).is_ok());
    let cfg = config_snapshot();
    assert_eq!(
        cfg.info_log_format.parts,
        LogFormatter::parse_from_string(info_fmt_txt).unwrap().parts
    );
    fs::remove_file(p).ok();
}

#[test]
fn ini_full_file_config_valid() {
    init();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let archive_dir_name = format!("test_ini_archives_full_{}", ts);
    let content = format!(
        "[Config]\n\
        file=app_{{date}}_{{time}}.log\n\
        compression=zip\n\
        rotations=\"1 day, 10 MB, 12:30\"\n\
        archive_dir={}\n",
        archive_dir_name
    );
    let p = temp_ini_file(&content);
    let load_res = load_config_from_file(p.to_str().unwrap());
    assert!(
        load_res.is_ok(),
        "load_config_from_file failed: {:?}",
        load_res.err()
    );

    let cfg = config_snapshot();
    assert!(cfg.file_manager.is_some());
    assert_eq!(cfg.archive_dir, Some(PathBuf::from(&archive_dir_name)));
    assert!(Path::new(&archive_dir_name).is_dir());

    let fm_lock = cfg.file_manager.as_ref().unwrap().lock().unwrap();
    let fm_dbg = format!("{:?}", fm_lock);
    assert!(fm_dbg.contains("Zip"));
    assert!(fm_dbg.contains("Period"));
    assert!(fm_dbg.contains("Size"));
    assert!(fm_dbg.contains("Time"));

    cleanup_archive_dir(&archive_dir_name);
    fs::remove_file(p).ok();
}

#[test]
fn ini_file_name_invalid_format() {
    init();
    let p = temp_ini_file("[Config]\nfile=test<bad>.log\n");
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(res, Err(ReadFromConfigFileError::SetFile(_))));
    fs::remove_file(p).ok();
}

#[test]
fn ini_compression_without_file_name() {
    init();
    let p = temp_ini_file("[Config]\ncompression=zip\n");
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
fn ini_invalid_compression_value() {
    init();
    let p = temp_ini_file("[Config]\nfile=app.log\ncompression=rar\n");
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
fn ini_rotations_without_file_name() {
    init();
    let p = temp_ini_file("[Config]\nrotations=1 day\n");
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
fn ini_invalid_rotation_value() {
    init();
    let p = temp_ini_file("[Config]\nfile=app.log\nrotations=invalid rotation\n");
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
fn ini_rotations_string_variants() {
    let valid_rotations = [
        ("1 day", 1),
        ("10 MB, 12:00", 2),
        ("1 week, 50 KB, 00:00", 3),
        ("  1 day  ,  10 MB  ", 2), // Check with spaces
    ];
    for (rot_str, expected_count) in valid_rotations {
        init();
        let content = format!("[Config]\nfile=app.log\nrotations={}\n", rot_str);
        let p = temp_ini_file(&content);
        let load_res = load_config_from_file(p.to_str().unwrap());
        assert!(
            load_res.is_ok(),
            "Failed for rotations='{}': {:?}",
            rot_str,
            load_res.err()
        );

        let cfg = config_snapshot();
        let fm_lock = cfg.file_manager.as_ref().unwrap().lock().unwrap();
        let fm_dbg = format!("{:?}", fm_lock);
        // A bit fragile, but count occurrences of "Rotation { rotation_type:"
        assert_eq!(
            fm_dbg.matches("Rotation { rotation_type:").count(),
            expected_count,
            "Rotation count mismatch for '{}'",
            rot_str
        );
        fs::remove_file(p).ok();
    }

    init(); // Empty rotation string
    let p_empty_rot = temp_ini_file("[Config]\nfile=app.log\nrotations=\n");
    let res_empty_rot = load_config_from_file(p_empty_rot.to_str().unwrap());
    // add_rotation("") fails
    assert!(matches!(
        res_empty_rot,
        Err(ReadFromConfigFileError::AddRotation(
            AddRotationError::IncorrectFormatGiven
        ))
    ));
    fs::remove_file(p_empty_rot).ok();
}

#[test]
fn ini_archive_dir_creates_directory() {
    init();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let archive_dir_name = format!("test_ini_archive_dir_creation_{}", ts);
    let content = format!("[Config]\narchive_dir={}\n", archive_dir_name);
    let p = temp_ini_file(&content);
    assert!(load_config_from_file(p.to_str().unwrap()).is_ok());
    assert_eq!(
        config_snapshot().archive_dir,
        Some(PathBuf::from(&archive_dir_name))
    );
    assert!(Path::new(&archive_dir_name).is_dir());
    cleanup_archive_dir(&archive_dir_name);
    fs::remove_file(p).ok();
}

#[test]
fn ini_archive_dir_path_is_file() {
    init();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_name = format!("test_ini_file_as_dir_{}", ts);
    File::create(&file_name)
        .unwrap()
        .write_all(b"file content")
        .unwrap();
    let content = format!("[Config]\narchive_dir={}\n", file_name);
    let p = temp_ini_file(&content);
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetArchiveDirError(_))
    ));
    fs::remove_file(file_name).ok();
    fs::remove_file(p).ok();
}

#[test]
fn ini_empty_file() {
    init();
    let p = temp_ini_file("");
    let res = load_config_from_file(p.to_str().unwrap());
    // Ini::load_from_file("") likely parse error or specific error for empty.
    // It gives ParseError("couldn't find Config section...") because section is checked after load.
    assert!(
        matches!(&res, Err(ReadFromConfigFileError::ParseError(s)) if s.contains("couldn't find Config section"))
    );
    fs::remove_file(p).ok();
}

#[test]
fn ini_malformed_file() {
    init();
    let p = temp_ini_file("[Config\nlevel=info\n"); // Malformed section
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(matches!(&res, Err(ReadFromConfigFileError::ParseError(_)))); // from ini::Error::Parse
    fs::remove_file(p).ok();
}

#[test]
fn ini_missing_config_section() {
    init();
    let p = temp_ini_file("level=info\n[OtherSection]\nkey=val\n");
    let res = load_config_from_file(p.to_str().unwrap());
    assert!(
        matches!(&res, Err(ReadFromConfigFileError::ParseError(s)) if s.contains("couldn't find Config section"))
    );
    fs::remove_file(p).ok();
}

#[test]
fn ini_fields_outside_config_section_ignored() {
    init();
    // level=info is not in [Config] so it should be ignored. Default level (INFO) should remain.
    let p = temp_ini_file("level=trace\n[Config]\ncolorized=true\n");
    assert!(load_config_from_file(p.to_str().unwrap()).is_ok());
    let cfg = config_snapshot();
    assert_eq!(cfg.level, Level::INFO); // Default, not trace
    assert!(cfg.colorized);
    fs::remove_file(p).ok();
}

#[test]
fn ini_unknown_fields_in_config_section_ignored() {
    init();
    let p = temp_ini_file("[Config]\nlevel=warn\nunknown_key=value\n");
    assert!(load_config_from_file(p.to_str().unwrap()).is_ok());
    assert_eq!(config_snapshot().level, Level::WARN);
    fs::remove_file(p).ok();
}

#[test]
fn ini_missing_file() {
    init();
    let bogus_path = "/no/such/path/to_ini_file.ini";
    let res = load_config_from_file(bogus_path);
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::ReadFileError(_))
    ));
}

#[test]
fn ini_incorrect_extension_but_valid_ini_content() {
    init();
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("loggit_ini_test_wrong_ext_{}.txt", ts));
    File::create(&path)
        .unwrap()
        .write_all(b"[Config]\nlevel=error\n")
        .unwrap();

    let res = load_config_from_file(path.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::IncorrectFileExtension)
    ));
    fs::remove_file(path).ok();
}

#[test]
fn ini_file_name_without_extension_in_path() {
    init();
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("loggit_ini_test_no_ext_{}", ts));

    let res = load_config_from_file(path.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::IncorrectFileName)
    ));
}
