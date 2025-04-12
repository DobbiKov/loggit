use thiserror::Error;

use super::file_handler::file_manager::FileManagerFromStringError;

#[derive(Error, Debug)]
pub enum SetFileError {
    #[error("unable to load config")]
    UnableToLoadConfig,
    #[error("unable to load from string: {0}")]
    UnableToLoadFromString(FileManagerFromStringError),
}

#[derive(Error, Debug)]
pub enum SetCompressionError {
    #[error("unable to load config")]
    UnableToLoadConfig,
    #[error("a file isn't set")]
    FileIsntSet,
}

#[derive(Error, Debug)]
pub enum AddRotationError {
    #[error("unable to load config")]
    UnableToLoadConfig,
    #[error("a file isn't set")]
    FileIsntSet,
    #[error("incorrect format given")]
    IncorrectFormatGiven,
}

#[derive(Error, Debug)]
pub enum SetLogLevelError {
    #[error("unable to load config")]
    UnableToLoadConfig,
}
#[derive(Error, Debug)]
pub enum SetPrintToTerminalError {
    #[error("unable to load config")]
    UnableToLoadConfig,
}

#[derive(Error, Debug)]
pub enum SetColorizedError {
    #[error("unable to load config")]
    UnableToLoadConfig,
}

#[derive(Error, Debug)]
pub enum SetLevelFormattingError {
    #[error("unable to load config")]
    UnableToLoadConfig,
    #[error("incorrect formatting")] // TODO!
    IncorrectFormatGiven,
}
