// tests/multithread_logging.rs
//
// Run with:  cargo test --test multithread_logging
//
// These are *integration* tests (compiled as a separate crate) so we use the
// public API only.

use std::{fs, thread, time::Duration};

use loggit::{
    info,
    logger::{init, set_file, set_log_level, set_print_to_terminal},
    Level,
};

/// Number of worker threads and messages per thread.
const THREADS: usize = 8;
const MSG_PER_THREAD: usize = 200;

/// Returns the first path in `.` that matches our test‑file prefix.
fn find_log_file(prefix: &str) -> std::path::PathBuf {
    fs::read_dir(".")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(prefix) && n.ends_with(".txt"))
                .unwrap_or(false)
        })
        .expect("log file not found")
}

/// A helper to remove the generated log file at the end.
fn cleanup(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}

/// Test that many threads can append to the *same* log file without losing data.
#[test]
fn concurrent_file_logging() {
    // 1.  Fresh logger state for every test invocation.
    init();
    set_print_to_terminal(false).unwrap();
    set_log_level(Level::INFO).unwrap();

    // Unique prefix so parallel test runs don’t clash.
    let prefix = format!("mt_log_{}", chrono::Utc::now().timestamp_nanos());
    let pattern = format!("{prefix}_{{date}}_{{time}}.txt");
    set_file(&pattern).unwrap();

    // 2.  Spawn the workers.
    let mut handles = Vec::with_capacity(THREADS);
    for id in 0..THREADS {
        handles.push(thread::spawn(move || {
            for n in 0..MSG_PER_THREAD {
                info!("thread={id} line={n}");
            }
        }));
    }
    for h in handles {
        h.join().expect("worker panicked");
    }

    // 3.  Verify the file contains exactly the expected number of lines.
    let log_path = find_log_file(&prefix);
    let contents = fs::read_to_string(&log_path).expect("unable to read log file");
    let line_count = contents.lines().count();
    assert_eq!(
        line_count,
        THREADS * MSG_PER_THREAD,
        "expected {} lines, got {}",
        THREADS * MSG_PER_THREAD,
        line_count
    );

    cleanup(&log_path);
}

/// Flip the log level from multiple threads while logging – a smoke test for
/// the RwLock‑protected CONFIG under contention.
#[test]
fn concurrent_level_switching() {
    init();
    set_print_to_terminal(false).unwrap();

    let prefix = format!("level_flip_{}", chrono::Utc::now().timestamp_nanos());
    set_file(&format!("{prefix}_{{date}}_{{time}}.txt")).unwrap();

    // Spawn threads that either log or change the level.
    let mut handles = Vec::with_capacity(THREADS);

    // Half the threads just log.
    for id in 0..(THREADS / 2) {
        handles.push(thread::spawn(move || {
            for i in 0..500 {
                info!("logging‑thread={id} i={i}");
            }
        }));
    }

    // The other half keep toggling the level.
    let levels = [
        Level::TRACE,
        Level::DEBUG,
        Level::INFO,
        Level::WARN,
        Level::ERROR,
    ];
    handles.push(thread::spawn(move || {
        for lvl in levels.iter().cycle().take(1000) {
            set_log_level(*lvl).unwrap();
            // tiny sleep so other threads get to run
            thread::sleep(Duration::from_millis(1));
        }
    }));

    for h in handles {
        h.join().expect("thread panicked");
    }

    let log_path = find_log_file(&prefix);
    let bytes = fs::read(&log_path).expect("could not read file");
    assert!(!bytes.is_empty(), "file should contain at least some lines");

    cleanup(&log_path);
}
