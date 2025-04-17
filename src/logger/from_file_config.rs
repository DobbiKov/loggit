// this module aims to provide a feature of setting the config up from a file (without explicitely
// precising it in the file (but make it still possible))

use crate::Level;

use crate::logger;
use env_file_reader;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadFromConfigFileError {
    #[error("couldn't open the config file to read: {0}")]
    ReadFileError(std::io::Error),
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
}

pub fn read_from_env_file(path: &str) -> Result<(), ReadFromConfigFileError> {
    let vars_r = match env_file_reader::read_file(path) {
        Ok(f) => f,
        Err(e) => {
            return Err(ReadFromConfigFileError::ReadFileError(e));
        }
    };

    //enabled
    match vars_r.get("enabled") {
        None => {}
        Some(v) => match v.as_str() {
            "true" => {}
            "false" => return Err(ReadFromConfigFileError::DisabledToBeUsed),
            _ => return Err(ReadFromConfigFileError::IncorrectValue),
        },
    };

    if let Some(v) = vars_r.get("level") {
        match v.to_lowercase().as_str() {
            "trace" => logger::set_log_level(Level::TRACE)?,
            "debug" => logger::set_log_level(Level::DEBUG)?,
            "info" => logger::set_log_level(Level::INFO)?,
            "warn" => logger::set_log_level(Level::WARN)?,
            "error" => logger::set_log_level(Level::ERROR)?,
            _ => return Err(ReadFromConfigFileError::IncorrectValue),
        };
    }

    if let Some(v) = vars_r.get("print_to_terminal") {
        match v.as_str() {
            "true" => logger::set_print_to_terminal(true)?,
            "false" => logger::set_print_to_terminal(false)?,
            _ => return Err(ReadFromConfigFileError::IncorrectValue),
        };
    };

    if let Some(v) = vars_r.get("colorized") {
        match v.as_str() {
            "true" => logger::set_colorized(true)?,
            "false" => logger::set_colorized(false)?,
            _ => return Err(ReadFromConfigFileError::IncorrectValue),
        };
    };

    if let Some(v) = vars_r.get("global_formatting") {
        logger::set_global_formatting(v)?;
    }
    if let Some(v) = vars_r.get("trace_formatting") {
        logger::set_level_formatting(Level::TRACE, v)?;
    }
    if let Some(v) = vars_r.get("debug_formatting") {
        logger::set_level_formatting(Level::DEBUG, v)?;
    }
    if let Some(v) = vars_r.get("info_formatting") {
        logger::set_level_formatting(Level::INFO, v)?;
    }
    if let Some(v) = vars_r.get("warn_formatting") {
        logger::set_level_formatting(Level::WARN, v)?;
    }
    if let Some(v) = vars_r.get("error_formatting") {
        logger::set_level_formatting(Level::ERROR, v)?;
    }

    if let Some(v) = vars_r.get("file") {
        logger::set_file(v)?;
    }
    if let Some(v) = vars_r.get("compression") {
        logger::set_compression(v)?;
    }
    if let Some(v) = vars_r.get("rotations") {
        if !v.contains(',') {
            logger::add_rotation(v)?;
        } else {
            let rotations = v.split(',');
            for rot in rotations {
                logger::add_rotation(rot)?;
            }
        }
    }

    Ok(())
}

fn read_from_json_file(path: &str) {}

fn read_from_ini_file(path: &str) {}
