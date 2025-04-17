use thiserror::Error;

use super::{
    file_handler::file_manager::FileManagerFromStringError, formatter::ParseStringToWrappersError,
};

#[derive(Debug, thiserror::Error)]
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
