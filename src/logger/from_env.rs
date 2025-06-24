//! Load configuration options from environment variables.
//!
//! The variables follow the same naming as the configuration file fields, such
//! as `level`, `print_to_terminal` or `file_name`.

use std::env;

use crate::logger::from_file_config::{parse_inter_config_from_serde_config, ConfigForSerde};
use crate::logger::set_errors::ReadFromConfigFileError;

fn parse_config_from_env() -> Result<ConfigForSerde, ReadFromConfigFileError> {
    let mut res_conf: ConfigForSerde = Default::default();

    if let Ok(v) = env::var("level") {
        res_conf.level = Some(v.to_owned())
    }

    if let Ok(v) = env::var("print_to_terminal") {
        res_conf.print_to_terminal = Some(v.to_owned());
    };

    if let Ok(v) = env::var("colorized") {
        res_conf.colorized = Some(v.to_owned());
    };

    if let Ok(v) = env::var("global_formatting") {
        res_conf.global_formatting = Some(v.to_owned());
    }
    if let Ok(v) = env::var("trace_formatting") {
        res_conf.trace_formatting = Some(v.to_owned());
    }
    if let Ok(v) = env::var("debug_formatting") {
        res_conf.debug_formatting = Some(v.to_owned());
    }
    if let Ok(v) = env::var("info_formatting") {
        res_conf.info_formatting = Some(v.to_owned());
    }
    if let Ok(v) = env::var("warn_formatting") {
        res_conf.warn_formatting = Some(v.to_owned());
    }
    if let Ok(v) = env::var("error_formatting") {
        res_conf.error_formatting = Some(v.to_owned());
    }

    if let Ok(v) = env::var("file_name") {
        res_conf.file_name = Some(v.to_owned());
    }
    if let Ok(v) = env::var("compression") {
        res_conf.compression = Some(v.to_owned());
    }
    if let Ok(v) = env::var("archive_dir") {
        res_conf.archive_dir = Some(v.to_owned());
    }
    if let Ok(v) = env::var("rotations") {
        let mut rots = Vec::<String>::new();
        if !v.contains(',') {
            rots.push(v.to_owned());
        } else {
            let rotations = v.split(',');
            for rot in rotations {
                let rot = rot.trim_start().trim_end();
                rots.push(rot.to_string());
            }
        }
        res_conf.rotations = Some(rots);
    }
    Ok(res_conf)
}

/// Read environment variables and apply them to the logger configuration.
pub(crate) fn load_config_from_env() -> Result<(), ReadFromConfigFileError> {
    let serde_conf = parse_config_from_env()?;
    let inter_conf = parse_inter_config_from_serde_config(serde_conf)
        .map_err(|e| ReadFromConfigFileError::ParseError(e.to_string()))?;

    inter_conf.apply()
}
