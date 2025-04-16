//! Comprehensive tests for `logger::from_file_config::read_from_env_file`
//! covering every supported key (valid & invalid cases) and verifying the
//! effect on the global `CONFIG`.
//!
//! Add `mod from_file_config;` to `src/tests/mod.rs` to include these.

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    logger::{
        from_file_config::{read_from_env_file, ReadFromConfigFileError},
        init,
    },
    Level, CONFIG,
};

use crate::logger::formatter::LogFormatter;

/// Writes `contents` to a unique temporary file and returns its [`PathBuf`].
fn temp_env_file(contents: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    path.push(format!("loggit_env_test_{}.env", ts));

    let mut file = File::create(&path).expect("failed to create temp env‑file");
    write!(file, "{}", contents).expect("failed to write temp env‑file");
    path
}

/// Helper that returns a cloned snapshot of CONFIG after acquiring a read lock.
fn config_snapshot() -> crate::Config {
    CONFIG
        .read()
        .unwrap()
        .as_ref()
        .expect("CONFIG should be initialised")
        .clone()
}

#[test]
fn env_enabled_variants() {
    // enabled=false → DisabledToBeUsed
    init();
    let p = temp_env_file("enabled=false\n");
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::DisabledToBeUsed)
    ));
    fs::remove_file(p).ok();

    // enabled missing (not set) should simply work (Ok)
    init();
    let p = temp_env_file("level=info\n");
    assert!(read_from_env_file(p.to_str().unwrap()).is_ok());
    fs::remove_file(p).ok();

    // enabled=invalid → IncorrectValue
    init();
    let p = temp_env_file("enabled=maybe\n");
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(res, Err(ReadFromConfigFileError::IncorrectValue)));
    fs::remove_file(p).ok();
}

#[test]
fn env_all_levels_valid() {
    let cases = [
        ("trace", Level::TRACE),
        ("debug", Level::DEBUG),
        ("info", Level::INFO),
        ("warn", Level::WARN),
        ("error", Level::ERROR),
    ];

    for (lvl_str, lvl_enum) in cases {
        init();
        let p = temp_env_file(&format!("enabled=true\nlevel={}\n", lvl_str));
        assert!(read_from_env_file(p.to_str().unwrap()).is_ok());
        let cfg = config_snapshot();
        assert_eq!(cfg.level, lvl_enum, "level `{}` not applied", lvl_str);
        fs::remove_file(p).ok();
    }

    // invalid level value
    init();
    let p = temp_env_file("level=verbose\n");
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(res, Err(ReadFromConfigFileError::IncorrectValue)));
    fs::remove_file(p).ok();
}

#[test]
fn env_print_to_terminal_variants() {
    // true
    init();
    let p = temp_env_file("print_to_terminal=true\n");
    assert!(read_from_env_file(p.to_str().unwrap()).is_ok());
    assert!(config_snapshot().print_to_terminal);
    fs::remove_file(p).ok();

    // false
    init();
    let p = temp_env_file("print_to_terminal=false\n");
    assert!(read_from_env_file(p.to_str().unwrap()).is_ok());
    assert!(!config_snapshot().print_to_terminal);
    fs::remove_file(p).ok();

    // invalid → IncorrectValue
    init();
    let p = temp_env_file("print_to_terminal=nope\n");
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(res, Err(ReadFromConfigFileError::IncorrectValue)));
    fs::remove_file(p).ok();
}

#[test]
fn env_colorized_variants() {
    // true
    init();
    let p = temp_env_file("colorized=true\n");
    assert!(read_from_env_file(p.to_str().unwrap()).is_ok());
    assert!(config_snapshot().colorized);
    fs::remove_file(p).ok();

    // false
    init();
    let p = temp_env_file("colorized=false\n");
    assert!(read_from_env_file(p.to_str().unwrap()).is_ok());
    assert!(!config_snapshot().colorized);
    fs::remove_file(p).ok();

    // invalid
    init();
    let p = temp_env_file("colorized=rainbow\n");
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(res, Err(ReadFromConfigFileError::IncorrectValue)));
    fs::remove_file(p).ok();
}

#[test]
fn env_global_formatting_ok_and_bad() {
    // Valid global formatting – should update every level formatter
    init();
    let fmt_txt = "[{level}] {message}";
    let p = temp_env_file(&format!("global_formatting=\"{}\"\n", fmt_txt));
    assert!(read_from_env_file(p.to_str().unwrap()).is_ok());

    let cfg = config_snapshot();
    let expected = LogFormatter::parse_from_string(fmt_txt).unwrap();
    assert_eq!(cfg.trace_log_format.parts, expected.parts);
    assert_eq!(cfg.debug_log_format.parts, expected.parts);
    assert_eq!(cfg.error_log_format.parts, expected.parts);
    fs::remove_file(p).ok();

    // Invalid formatting – unmatched brackets → SetLevelFormatting error
    init();
    let bad_fmt = "<red>[{level}]"; // missing closing <red>
    let p = temp_env_file(&format!("global_formatting={}\n", bad_fmt));
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetLevelFormatting(_))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn env_individual_level_formatting_invalid() {
    // invalid debug_formatting should surface the same SetLevelFormatting error
    init();
    let p = temp_env_file("debug_formatting=<blue>{message}\n"); // missing closing <blue>
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetLevelFormatting(_))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn env_file_and_compression_and_rotations() {
    // Everything valid
    init();
    let p = temp_env_file(
        "file=app_{date}_{time}.txt\ncompression=zip\nrotations=\"1 day,500 MB,12:30\"\n",
    );
    assert!(read_from_env_file(p.to_str().unwrap()).is_ok());

    let cfg = config_snapshot();
    assert!(
        cfg.file_manager.is_some(),
        "file_manager should be configured"
    );
    let fm_dbg = format!("{:?}", cfg.file_manager.as_ref().unwrap());
    assert!(
        fm_dbg.contains("rotation: ["),
        "rotations should be present"
    );
    fs::remove_file(p).ok();
}

#[test]
fn env_file_invalid_format() {
    init();
    // forbidden character '<' inside file format
    let p = temp_env_file("file=bad<name>.txt\n");
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(res, Err(ReadFromConfigFileError::SetFile(_))));
    fs::remove_file(p).ok();
}

#[test]
fn env_compression_without_file() {
    init();
    let p = temp_env_file("compression=zip\n"); // no file configured first
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetCompression(_))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn env_invalid_compression_value() {
    init();
    // configure file properly, but give unsupported compression algorithm
    let p = temp_env_file("file=app_{date}.txt\ncompression=rar\n");
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::SetCompression(_))
    ));
    fs::remove_file(p).ok();
}

#[test]
fn env_rotations_invalid() {
    init();
    let p = temp_env_file("file=app_{date}.txt\nrotations=invalid\n");
    let res = read_from_env_file(p.to_str().unwrap());
    assert!(matches!(res, Err(ReadFromConfigFileError::AddRotation(_))));
    fs::remove_file(p).ok();
}

#[test]
fn env_missing_file() {
    init();
    let bogus_path = "/no/such/path/to_env_file.loggit";
    let res = read_from_env_file(bogus_path);
    assert!(matches!(
        res,
        Err(ReadFromConfigFileError::ReadFileError(_))
    ));
}
