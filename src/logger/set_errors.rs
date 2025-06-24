//! Error types used throughout the configuration helpers.

use crate::logger;
use thiserror::Error;

use super::{
    file_handler::file_manager::FileManagerFromStringError, formatter::ParseStringToWrappersError,
};

#[derive(Debug, thiserror::Error)]
/// Errors while accessing or mutating the global configuration.
pub enum AccessError {
    #[error("unable to load config")]
    LoadConfig,
    #[error("file isn’t set")]
    FileNotSet,
}

#[derive(Error, Debug)]
pub enum SetFileError {
    #[error("unable to load config")]
    UnableToLoadConfig,
    #[error("unable to load from string: {0}")]
    UnableToLoadFromString(FileManagerFromStringError),
    #[error("the file isn't set")]
    FileIsntSet,
}

#[derive(Error, Debug)]
pub enum SetCompressionError {
    #[error("unable to load config")]
    UnableToLoadConfig,
    #[error("a file isn't set")]
    FileIsntSet,
    #[error("incorrect compression value")]
    IncorrectCompressionValue,
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
    IncorrectFormatGiven(ParseStringToWrappersError),
}

#[derive(Error, Debug)]
pub enum SetArchiveDirError {
    #[error("unable to load config")]
    UnableToLoadConfig,
    #[error("unable to create dir {0}")]
    UnableToCreateDir(#[from] std::io::Error),
}

impl From<ParseStringToWrappersError> for SetLevelFormattingError {
    fn from(value: ParseStringToWrappersError) -> Self {
        SetLevelFormattingError::IncorrectFormatGiven(value)
    }
}

impl From<AccessError> for SetCompressionError {
    fn from(e: AccessError) -> Self {
        match e {
            AccessError::LoadConfig => SetCompressionError::UnableToLoadConfig,
            AccessError::FileNotSet => SetCompressionError::FileIsntSet,
        }
    }
}

impl From<AccessError> for SetFileError {
    fn from(e: AccessError) -> Self {
        match e {
            AccessError::LoadConfig => SetFileError::UnableToLoadConfig,
            AccessError::FileNotSet => SetFileError::FileIsntSet,
        }
    }
}

impl From<AccessError> for AddRotationError {
    fn from(e: AccessError) -> Self {
        match e {
            AccessError::LoadConfig => AddRotationError::UnableToLoadConfig,
            AccessError::FileNotSet => AddRotationError::FileIsntSet,
        }
    }
}

#[derive(Debug, Error)]
pub enum ReadFromConfigFileError {
    #[error("couldn't open the config file to read: {0}")]
    ReadFileError(std::io::Error),
    #[error("incorrect file name")]
    IncorrectFileName,
    #[error("incorrect file extension")]
    IncorrectFileExtension,
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("this config file is disabled to be used")]
    DisabledToBeUsed,
    #[error("incorrect value given")]
    IncorrectValue,

    #[error("failed to set log level: {0}")]
    SetLogLevel(#[from] logger::set_errors::SetLogLevelError),

    #[error("failed to set print_to_terminal: {0}")]
    SetPrintToTerminal(#[from] logger::set_errors::SetPrintToTerminalError),

    #[error("failed to set colorized: {0}")]
    SetColorized(#[from] logger::set_errors::SetColorizedError),

    #[error("failed to set global formatting: {0}")]
    SetLevelFormatting(#[from] logger::set_errors::SetLevelFormattingError),

    #[error("failed to configure file output: {0}")]
    SetFile(#[from] logger::set_errors::SetFileError),

    #[error("failed to set compression: {0}")]
    SetCompression(#[from] logger::set_errors::SetCompressionError),

    #[error("failed to add rotation: {0}")]
    AddRotation(#[from] logger::set_errors::AddRotationError),

    #[error("failed to set archive dir: {0}")]
    SetArchiveDirError(#[from] logger::set_errors::SetArchiveDirError),
}

#[derive(Debug, Error)]
pub enum ParseConfigError {
    #[error("incorrect value given")]
    IncorrectValue,
}
