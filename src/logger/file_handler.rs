//! Infrastructure for log file output.
//!
//! This module groups together helpers responsible for formatting log file
//! names, managing file rotation and compression, and writing log entries to
//! disk.

pub(crate) mod file_formatter;
pub(crate) mod file_manager;
pub(crate) mod file_name;

// FileName
