// tests/archivation_rotation.rs
//
// Checks that size‑based rotation really produces a .zip archive in the
// configured archive directory.

use std::{fs, thread, time::Duration};

use loggit::{
    info,
    logger::{
        add_rotation, init, set_archive_dir, set_compression, set_file, set_log_level,
        set_print_to_terminal,
    },
    Level,
};

/// We rotate after 1 KB, so a few hundred reasonably long messages are plenty.
const MESSAGES: usize = 50;

#[test]
fn rotation_creates_zip_archive() {
    // Fresh logger config.
    init();
    set_print_to_terminal(false).unwrap();
    set_log_level(Level::INFO).unwrap();

    // Unique prefixes so parallel `cargo test` jobs never clash.
    let ts = chrono::Utc::now().timestamp_nanos();
    let prefix = format!("ziprot_{ts}");
    let log_pattern = format!("{prefix}_{{date}}_{{time}}.log");

    // Use a disposable archive directory under the project root.
    let archive_dir = format!("arch_{ts}");
    set_archive_dir(&archive_dir).unwrap();

    // Configure file output + rotation + compression.
    set_file(&log_pattern).unwrap();
    set_compression("zip").unwrap();
    add_rotation("1 KB").unwrap(); // rotate as soon as file > 1024 B

    // Generate enough output to exceed 1 KB.
    for n in 0..MESSAGES {
        info!("msg {n}: lorem ipsum dolor sit amet, consectetur adipiscing elit.");
    }
    // One more message so a new log file gets created after rotation.
    info!("post-rotation message");

    // Tiny wait to make sure the last write flushed & compression finished.
    thread::sleep(Duration::from_millis(50));
    // And another log entry to guarantee the new file exists on disk.
    info!("post-check message");
    thread::sleep(Duration::from_millis(50));

    // Assert: at least one .zip archive exists in the archive directory.
    let mut zip_found = false;
    for entry in fs::read_dir(&archive_dir).expect("cannot read archive dir") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("zip") {
            zip_found = true;
            assert!(
                path.metadata().unwrap().len() > 0,
                "zip archive is unexpectedly empty"
            );
        }
    }
    assert!(zip_found, "no .zip archive produced after rotation");

    // Assert: rotation should leave a new active log file.
    let mut log_found = false;
    for entry in fs::read_dir(".").unwrap() {
        let p = entry.unwrap().path();
        if p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with(&prefix) && n.ends_with(".log"))
            .unwrap_or(false)
        {
            log_found = true;
        }
    }
    assert!(log_found, "rotation did not produce a new active log file");

    // ───── clean‑up ────────────────────────────────────────────────────────────
    // remove generated log files
    for entry in fs::read_dir(".").unwrap() {
        let p = entry.unwrap().path();
        if p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with(&prefix))
            .unwrap_or(false)
        {
            let _ = fs::remove_file(p);
        }
    }
    // remove the archive folder
    let _ = fs::remove_dir_all(&archive_dir);
}
