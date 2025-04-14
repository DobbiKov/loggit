use crate::logger::file_handler::file_manager::{CompressFileError, FileManager};
use crate::Config;
use crate::Level;
use std::fs;
use std::path::Path;

/// Helper to build a dummy configuration.
/// Adjust fields as necessary if your Config struct requires more fields.
fn dummy_config() -> Config {
    Config {
        level: Level::INFO,
        file_manager: None,
        ..Default::default()
    }
}

/// Helper to create a FileManager using a valid file format.
/// Panics if initialization fails.
fn get_dummy_file_manager() -> FileManager {
    let config = dummy_config();
    let fm_opt = FileManager::init_from_string("test_log_{date}_{time}.txt", config.clone());
    assert!(
        fm_opt.is_ok(),
        "FileManager initialization failed with a valid format"
    );
    fm_opt.unwrap()
}

#[test]
fn test_init_from_string_valid() {
    let config = dummy_config();
    let fm = FileManager::init_from_string("log_{date}_{time}.txt", config);
    assert!(
        fm.is_ok(),
        "Expected valid FileManager from a correct file format"
    );
}

#[test]
fn test_init_from_string_invalid() {
    let config = dummy_config();
    let fm = FileManager::init_from_string("log_{date}_{time}<.txt", config);
    assert!(
        fm.is_err(),
        "Expected failure when using forbidden characters in the format"
    );
}

#[test]
fn test_remove_rotations() {
    let mut fm = get_dummy_file_manager();
    // Add a valid rotation
    let added = fm.add_rotation("1 day");
    assert!(
        added,
        "Expected add_rotation to succeed for a valid rotation definition"
    );
    fm.remove_rotations();
    // Since internal rotation list is private, check via Debug formatting.
    let fm_debug = format!("{:?}", fm);
    assert!(
        fm_debug.contains("rotation: []"),
        "Expected rotations to be removed (empty list)"
    );
}

#[test]
fn test_add_rotation() {
    let mut fm = get_dummy_file_manager();
    // Valid rotation
    let valid = fm.add_rotation("1 day");
    assert!(valid, "Expected add_rotation to succeed with '1 day'");

    // Invalid rotation should return false.
    let invalid = fm.add_rotation("invalid");
    assert!(
        !invalid,
        "Expected add_rotation to fail with an invalid rotation string"
    );
}

#[test]
fn test_set_and_remove_compression() {
    let mut fm = get_dummy_file_manager();
    // Valid compression type ("zip")
    let set_ok = fm.set_compression("zip");
    assert!(set_ok, "Expected set_compression to accept 'zip'");
    // Remove compression
    fm.remove_compression();
    // Now, trying to compress should fail due to missing settings.
    let result = fm.compress_file("nonexistent.txt");
    match result {
        Err(CompressFileError::UnableToGetCompressionSettings) => {}
        _ => panic!("Expected an error for missing compression settings after removal"),
    }
}

#[test]
fn test_create_new_file() {
    let mut fm = get_dummy_file_manager();
    let config = dummy_config();
    // Create a new file.
    let res = fm.create_new_file(&config);
    assert!(res.is_ok(), "Expected create_new_file to succeed");

    let file_name = fm.get_file_name();
    assert!(
        Path::new(&file_name).exists(),
        "Created file does not exist on disk"
    );
    // Cleanup the generated file.
    let _ = fs::remove_file(&file_name);
}

#[test]
fn test_delete_file() {
    // Create a temporary file.
    let temp_file = "temp_test_file.txt";
    fs::File::create(temp_file).expect("Failed to create temporary file");
    assert!(Path::new(temp_file).exists(), "Temporary file should exist");
    // Delete it.
    let res = FileManager::delete_file(temp_file);
    assert!(res.is_ok(), "Expected delete_file to succeed");
    assert!(
        !Path::new(temp_file).exists(),
        "Temporary file should have been deleted"
    );
}

#[test]
fn test_write_log_success() {
    let mut fm = get_dummy_file_manager();
    let config = dummy_config();
    // Create a new log file.
    fm.create_new_file(&config)
        .expect("Expected file creation to succeed");
    let file_name = fm.get_file_name();

    // Write a log message.
    let log_message = "Test log message";
    let write_res = fm.write_log(log_message, config);
    assert!(write_res.is_ok(), "Expected write_log to succeed");

    // Verify the log file contains the message.
    let content = fs::read_to_string(&file_name).unwrap_or_else(|_| String::new());
    assert!(
        content.contains(log_message),
        "Log file does not contain the written message"
    );
    // Cleanup:
    let _ = fs::remove_file(&file_name);
}

#[test]
fn test_compress_file() {
    let mut fm = get_dummy_file_manager();
    let config = dummy_config();
    // Create a new file.
    fm.create_new_file(&config)
        .expect("Expected file creation to succeed");
    let file_name = fm.get_file_name();

    // Write some content to the file.
    fs::write(&file_name, "Dummy log content").expect("Failed to write dummy log content");

    // Set compression to trigger compression.
    assert!(
        fm.set_compression("zip"),
        "Expected setting compression to succeed"
    );
    let comp_res = fm.compress_file(&file_name);
    assert!(comp_res.is_ok(), "Expected compress_file to succeed");

    // Check that the zip archive was created in the compression folder.
    let zip_file = format!("./loggit_archives/{}.zip", file_name);
    assert!(
        Path::new(&zip_file).exists(),
        "Expected the zip archive to exist"
    );

    // Cleanup: remove both files and the archive folder.
    let _ = fs::remove_file(&file_name);
    let _ = fs::remove_file(&zip_file);
    let _ = fs::remove_dir_all("./loggit_archives/");
}
