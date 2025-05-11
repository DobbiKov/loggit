// this module aims to provide a feature of setting the config up from a file (without explicitely
// precising it in the file (but make it still possible))

use std::io::Read;

use crate::logger::set_errors::{ParseConfigError, ReadFromConfigFileError};
use crate::Level;

use crate::logger;
use env_file_reader;
use ini::Ini;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Default, Debug)]
struct ConfigForSerde {
    enabled: Option<String>,
    level: Option<String>,
    print_to_terminal: Option<String>,
    colorized: Option<String>,
    global_formatting: Option<String>,
    trace_formatting: Option<String>,
    debug_formatting: Option<String>,
    info_formatting: Option<String>,
    warn_formatting: Option<String>,
    error_formatting: Option<String>,

    file_name: Option<String>,
    compression: Option<String>,
    rotations: Option<Vec<String>>,
    archive_dir: Option<String>,
}

#[derive(Default)]
struct InterConfig {
    enabled: Option<bool>,
    level: Option<Level>,
    print_to_terminal: Option<bool>,
    colorized: Option<bool>,
    global_formatting: Option<String>,
    trace_formatting: Option<String>,
    debug_formatting: Option<String>,
    info_formatting: Option<String>,
    warn_formatting: Option<String>,
    error_formatting: Option<String>,

    file_name: Option<String>,
    compression: Option<String>,
    rotations: Option<Vec<String>>,
    archive_dir: Option<String>,
}

impl InterConfig {
    /// Apply all of the settings that were present in the parsed config.
    /// If `enabled` is Some(false), returns Err(DisabledToBeUsed) immediately.
    fn apply(self) -> Result<(), ReadFromConfigFileError> {
        // Honor the `enabled` flag
        if let Some(enabled) = self.enabled {
            if !enabled {
                return Err(ReadFromConfigFileError::DisabledToBeUsed);
            }
        }

        // Log level
        if let Some(level) = self.level {
            logger::set_log_level(level)?;
        }

        // Terminal output
        if let Some(to_term) = self.print_to_terminal {
            logger::set_print_to_terminal(to_term)?;
        }

        // Colorized output
        if let Some(col) = self.colorized {
            logger::set_colorized(col)?;
        }

        // Formatting
        if let Some(fmt) = self.global_formatting {
            logger::set_global_formatting(&fmt)?;
        }
        if let Some(fmt) = self.trace_formatting {
            logger::set_level_formatting(Level::TRACE, &fmt)?;
        }
        if let Some(fmt) = self.debug_formatting {
            logger::set_level_formatting(Level::DEBUG, &fmt)?;
        }
        if let Some(fmt) = self.info_formatting {
            logger::set_level_formatting(Level::INFO, &fmt)?;
        }
        if let Some(fmt) = self.warn_formatting {
            logger::set_level_formatting(Level::WARN, &fmt)?;
        }
        if let Some(fmt) = self.error_formatting {
            logger::set_level_formatting(Level::ERROR, &fmt)?;
        }

        // File output
        if let Some(pattern) = self.file_name {
            logger::set_file(&pattern)?;
        }
        if let Some(comp) = self.compression {
            logger::set_compression(&comp)?;
        }
        if let Some(dir) = self.archive_dir {
            // we ignore the returned PathBuf here
            let _ = logger::set_archive_dir(&dir)?;
        }
        if let Some(rotations) = self.rotations {
            for rot in rotations {
                logger::add_rotation(&rot)?;
            }
        }

        Ok(())
    }
}

impl TryFrom<ConfigForSerde> for InterConfig {
    type Error = ParseConfigError;

    fn try_from(value: ConfigForSerde) -> Result<Self, Self::Error> {
        let mut res_conf: InterConfig = Default::default();
        //enabled
        match value.enabled {
            None => {}
            Some(v) => match v.as_str() {
                "true" => res_conf.enabled = Some(true),
                "false" => res_conf.enabled = Some(false),
                _ => return Err(ParseConfigError::IncorrectValue),
            },
        };

        if let Some(v) = value.level {
            match v.to_lowercase().as_str() {
                "trace" => res_conf.level = Some(Level::TRACE),
                "debug" => res_conf.level = Some(Level::DEBUG),
                "info" => res_conf.level = Some(Level::INFO),
                "warn" => res_conf.level = Some(Level::WARN),
                "error" => res_conf.level = Some(Level::ERROR),
                _ => return Err(ParseConfigError::IncorrectValue),
            };
        }

        if let Some(v) = value.print_to_terminal {
            match v.as_str() {
                "true" => res_conf.print_to_terminal = Some(true),
                "false" => res_conf.print_to_terminal = Some(false),
                _ => return Err(ParseConfigError::IncorrectValue),
            };
        };

        if let Some(v) = value.colorized {
            match v.as_str() {
                "true" => res_conf.colorized = Some(true),
                "false" => res_conf.colorized = Some(false),
                _ => return Err(ParseConfigError::IncorrectValue),
            };
        };

        if let Some(v) = value.global_formatting {
            res_conf.global_formatting = Some(v);
        }
        if let Some(v) = value.trace_formatting {
            res_conf.trace_formatting = Some(v);
        }
        if let Some(v) = value.debug_formatting {
            res_conf.debug_formatting = Some(v);
        }
        if let Some(v) = value.info_formatting {
            res_conf.info_formatting = Some(v);
        }
        if let Some(v) = value.warn_formatting {
            res_conf.warn_formatting = Some(v);
        }
        if let Some(v) = value.error_formatting {
            res_conf.error_formatting = Some(v);
        }

        if let Some(v) = value.file_name {
            res_conf.file_name = Some(v);
        }
        if let Some(v) = value.compression {
            res_conf.compression = Some(v);
        }
        if let Some(v) = value.archive_dir {
            res_conf.archive_dir = Some(v);
        }
        if let Some(v) = value.rotations {
            res_conf.rotations = Some(v)
        }
        Ok(res_conf)
    }
}

fn parse_config_from_env_file(path: &str) -> Result<ConfigForSerde, ReadFromConfigFileError> {
    let mut res_conf: ConfigForSerde = Default::default();

    let vars_r = match env_file_reader::read_file(path) {
        Ok(f) => f,
        Err(e) => {
            return Err(ReadFromConfigFileError::ReadFileError(e));
        }
    };

    //enabled
    match vars_r.get("enabled") {
        None => {}
        Some(v) => {
            res_conf.enabled = Some(v.to_owned());
        }
    };

    if let Some(v) = vars_r.get("level") {
        res_conf.level = Some(v.to_owned())
    }

    if let Some(v) = vars_r.get("print_to_terminal") {
        res_conf.print_to_terminal = Some(v.to_owned());
    };

    if let Some(v) = vars_r.get("colorized") {
        res_conf.colorized = Some(v.to_owned());
    };

    if let Some(v) = vars_r.get("global_formatting") {
        res_conf.global_formatting = Some(v.to_owned());
    }
    if let Some(v) = vars_r.get("trace_formatting") {
        res_conf.trace_formatting = Some(v.to_owned());
    }
    if let Some(v) = vars_r.get("debug_formatting") {
        res_conf.debug_formatting = Some(v.to_owned());
    }
    if let Some(v) = vars_r.get("info_formatting") {
        res_conf.info_formatting = Some(v.to_owned());
    }
    if let Some(v) = vars_r.get("warn_formatting") {
        res_conf.warn_formatting = Some(v.to_owned());
    }
    if let Some(v) = vars_r.get("error_formatting") {
        res_conf.error_formatting = Some(v.to_owned());
    }

    if let Some(v) = vars_r.get("file") {
        res_conf.file_name = Some(v.to_owned());
    }
    if let Some(v) = vars_r.get("compression") {
        res_conf.compression = Some(v.to_owned());
    }
    if let Some(v) = vars_r.get("archive_dir") {
        res_conf.archive_dir = Some(v.to_owned());
    }
    if let Some(v) = vars_r.get("rotations") {
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

fn parse_config_from_json_file(path: &str) -> Result<ConfigForSerde, ReadFromConfigFileError> {
    let mut file = std::fs::File::open(path).map_err(ReadFromConfigFileError::ReadFileError)?;
    let mut contents = String::new();
    let read_res = file
        .read_to_string(&mut contents)
        .map_err(ReadFromConfigFileError::ReadFileError)?;

    let cfg: ConfigForSerde = serde_json::from_str::<ConfigForSerde>(&contents)
        .map_err(|e| ReadFromConfigFileError::ParseError(e.to_string()))?;
    Ok(cfg)
}

// temp pub
fn parse_config_from_ini_file(path: &str) -> Result<ConfigForSerde, ReadFromConfigFileError> {
    let mut res_conf: ConfigForSerde = Default::default();
    let conf = match Ini::load_from_file(path) {
        Err(e) => {
            return match e {
                ini::Error::Io(error) => Err(ReadFromConfigFileError::ReadFileError(error)),
                ini::Error::Parse(parse_error) => {
                    Err(ReadFromConfigFileError::ParseError(parse_error.to_string()))
                }
            }
        }
        Ok(r) => r,
    };
    // TODO
    let section = match conf.section(Some("Config")) {
        None => {
            return Err(ReadFromConfigFileError::ParseError(
                "couldn't find Config section in the provided ini file".to_string(),
            ))
        }
        Some(r) => r,
    };

    //enabled
    match section.get("enabled") {
        None => {}
        Some(v) => {
            res_conf.enabled = Some(v.to_owned());
        }
    };

    if let Some(v) = section.get("level") {
        res_conf.level = Some(v.to_owned())
    }

    if let Some(v) = section.get("print_to_terminal") {
        res_conf.print_to_terminal = Some(v.to_owned());
    };

    if let Some(v) = section.get("colorized") {
        res_conf.colorized = Some(v.to_owned());
    };

    if let Some(v) = section.get("global_formatting") {
        res_conf.global_formatting = Some(v.to_owned());
    }
    if let Some(v) = section.get("trace_formatting") {
        res_conf.trace_formatting = Some(v.to_owned());
    }
    if let Some(v) = section.get("debug_formatting") {
        res_conf.debug_formatting = Some(v.to_owned());
    }
    if let Some(v) = section.get("info_formatting") {
        res_conf.info_formatting = Some(v.to_owned());
    }
    if let Some(v) = section.get("warn_formatting") {
        res_conf.warn_formatting = Some(v.to_owned());
    }
    if let Some(v) = section.get("error_formatting") {
        res_conf.error_formatting = Some(v.to_owned());
    }

    if let Some(v) = section.get("file") {
        res_conf.file_name = Some(v.to_owned());
    }
    if let Some(v) = section.get("compression") {
        res_conf.compression = Some(v.to_owned());
    }
    if let Some(v) = section.get("archive_dir") {
        res_conf.archive_dir = Some(v.to_owned());
    }
    if let Some(v) = section.get("rotations") {
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

fn parse_config_file(path: &str) -> Result<ConfigForSerde, ReadFromConfigFileError> {
    if !path.contains(".") {
        return Err(ReadFromConfigFileError::IncorrectFileName);
    }

    let ext = match path.split(".").last() {
        None => return Err(ReadFromConfigFileError::IncorrectFileName),
        Some(r) => r,
    };

    match ext {
        "ini" => parse_config_from_ini_file(path),
        "json" => parse_config_from_json_file(path),
        "env" => parse_config_from_env_file(path),
        _ => Err(ReadFromConfigFileError::IncorrectFileExtension),
    }
}

fn parse_inter_config_from_serde_config(
    s_conf: ConfigForSerde,
) -> Result<InterConfig, ParseConfigError> {
    s_conf.try_into()
}

pub(crate) fn load_config_from_file(path: &str) -> Result<(), ReadFromConfigFileError> {
    let parse_conf = parse_config_file(path)?;
    let inter_conf = parse_inter_config_from_serde_config(parse_conf)
        .map_err(|e| ReadFromConfigFileError::ParseError(e.to_string()))?;
    inter_conf.apply()
}

fn read_from_json_file(path: &str) {}

fn read_from_ini_file(path: &str) {}
