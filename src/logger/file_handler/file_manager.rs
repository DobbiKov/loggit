use crate::Config;

use super::{file_formatter::FileFormatter, file_name::FileName};

pub(crate) struct FileManager {
    file_format: FileFormatter,
    file_name: FileName,
}

pub(crate) enum RotationType {
    Period(u32),  // every 1 week for example
    Time(u8, u8), //every day at 12:00 for example
    Size(u64),    //500 MB for example
}

impl RotationType {
    pub(crate) fn try_from_string(text: String) -> Option<RotationType> {
        if text.contains(":") {
            // time
            let sp: Vec<&str> = text.split(":").collect();
            if sp.len() != 2 {
                return None;
            }
            let h: u8 = match sp[0].parse() {
                Ok(n) => n,
                Err(_) => return None,
            };
            let m: u8 = match sp[1].parse() {
                Ok(n) => n,
                Err(_) => return None,
            };
            if !(0..=23).contains(&h) {
                return None;
            }
            if !(0..=59).contains(&m) {
                return None;
            }
            Some(RotationType::Time(h, m))
        } else if text.ends_with(" KB")
            || text.ends_with(" MB")
            || text.ends_with(" GB")
            || text.ends_with(" TB")
        {
            //size
            let t_len = text.len();
            let mut text = &text[0..(t_len - 3)];
            let num: u64 = match text.parse() {
                Ok(n) => n,
                Err(_) => {
                    return None;
                }
            };
            Some(RotationType::Size(num))
        } else if text.ends_with(" hour")
            || text.ends_with(" day")
            || text.ends_with(" week")
            || text.ends_with(" month")
            || text.ends_with(" year")
        {
            // period
            let finish_txt: &str = {
                if text.ends_with(" hour") {
                    " hour"
                } else if text.ends_with(" day") {
                    " day"
                } else if text.ends_with(" week") {
                    " week"
                } else if text.ends_with(" month") {
                    " month"
                } else {
                    " year"
                }
            };
            let fin_len = finish_txt.len();
            let str_len = text.len();
            let text_to_parse = &text[0..(str_len - fin_len)];
            let num: u32 = match text_to_parse.parse() {
                Ok(n) => n,
                Err(_) => {
                    return None;
                }
            };
            Some(RotationType::Period(num))
        } else {
            None
        }
    }
}

pub(crate) struct Rotation {
    rotation_type: RotationType,
    next_rotation: u64,
}

pub(crate) enum CompressionType {
    Zip,
}

pub(crate) struct FileConstraints {
    compression: Option<CompressionType>,
    rotation: Vec<Rotation>,
}

impl FileManager {
    pub(crate) fn init_from_string(format: String, config: Config) -> Option<FileManager> {
        let f_format = match FileFormatter::try_from_string(format) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("An error occured during parsing your format: {}", e);
                return None;
            }
        };
        let f_name = match FileName::from_file_formatter(f_format.clone(), config.level) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("An error occured during parsing your format: {}", e);
                return None;
            }
        };
        Some(FileManager {
            file_format: f_format,
            file_name: f_name,
        })
    }
    pub(crate) fn get_file_name(&self) -> String {
        todo!()
    }
}
