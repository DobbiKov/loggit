use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::Level;

use super::formatter::{LogFormatter, LogPart};

#[derive(Clone)]
pub(crate) struct FileFormatter {
    format: Vec<LogPart>,
}

#[derive(Debug)]
pub(crate) enum FileFormatterTryFromStringError {
    IncorrectCaracterGiven(char),
    EmptyStringGiven,
    NoFileExtensionProvided,
    IncorrectFormatPartGiven,
}
impl Display for FileFormatterTryFromStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mess = match self {
            FileFormatterTryFromStringError::IncorrectCaracterGiven(ch) => {
                format!("An incrorrect caracter given: {}", ch)
            }
            FileFormatterTryFromStringError::EmptyStringGiven => {
                "An empty string was provided!".to_string()
            }
            FileFormatterTryFromStringError::NoFileExtensionProvided => {
                "No file extension provided".to_string()
            }
            FileFormatterTryFromStringError::IncorrectFormatPartGiven => {
                "An incrorrect part was provided".to_string()
            }
        };
        write!(f, "{}", mess)
    }
}

impl FileFormatter {
    pub(crate) fn is_part_authorized(part: &LogPart) -> bool {
        match part {
            LogPart::Message | LogPart::File | LogPart::Line => false,
            _ => true,
        }
    }
    // pub(crate)
    pub fn get_file_name(&self, level: Level) -> String {
        let (year, month, day, hour, minute, second) = crate::helper::seconds_to_ymdhms(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        let mut res = String::new();
        for part in &self.format {
            let temp = match part {
                LogPart::Time => &format!("{}:{}:{}", hour, minute, second),
                LogPart::Date => &format!("{}-{}-{}", day, month, year),
                LogPart::Level => &level.to_string(),
                LogPart::Text(tt) => tt,
                _ => panic!("Incorrect file format given!"),
            };
            res.push_str(temp);
        }
        res
    }
    fn forbidden_caracters() -> [char; 4] {
        ['<', '>', '&', '%']
    }
    pub(crate) fn try_from_string(
        format: String,
    ) -> Result<FileFormatter, FileFormatterTryFromStringError> {
        for ch in FileFormatter::forbidden_caracters() {
            if format.contains(ch) {
                return Err(FileFormatterTryFromStringError::IncorrectCaracterGiven(ch));
            }
        }
        let elems = crate::logger::formatter::parse_string_to_logparts(format);
        let last_elem = match elems.last() {
            None => return Err(FileFormatterTryFromStringError::EmptyStringGiven),
            Some(el) => el,
        };
        let text = match &last_elem {
            super::formatter::LogPart::Text(t) => t,
            _ => return Err(FileFormatterTryFromStringError::NoFileExtensionProvided),
        };
        if !text.contains(".") {
            return Err(FileFormatterTryFromStringError::NoFileExtensionProvided);
        }
        if text.len() == 1 || text.ends_with(".") {
            return Err(FileFormatterTryFromStringError::NoFileExtensionProvided);
        }
        for elem in &elems {
            if !FileFormatter::is_part_authorized(&elem) {
                return Err(FileFormatterTryFromStringError::IncorrectFormatPartGiven);
            }
        }
        Ok(FileFormatter { format: elems })
    }
}
