use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{helper, Level};

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

// FileName
#[derive(Debug)]
pub(crate) struct FileName {
    file_name: String,
    file_num: Option<u32>,
    file_extension: String,
}

#[derive(Debug)]
pub(crate) enum FileNameFromFileFormatterError {
    NoFormatProvided,
    IncorrectLastPart,
    NoFileExtensionProvided,
    IncorrectFileExtension,
}
impl FileName {
    fn acceptable_file_extensions() -> Vec<String> {
        vec!["txt", "log"]
            .into_iter()
            .map(|x| x.to_string())
            .collect()
    }
    fn is_acceptable_file_extension<'a>(ext: &'a str) -> bool {
        FileName::acceptable_file_extensions().contains(&ext.to_string())
    }
    pub(crate) fn increase_num(&mut self) {
        match self.file_num {
            None => self.file_num = Some(1),
            Some(num) => self.file_num = Some(num + 1),
        };
    }
    // pub(crate)
    pub fn get_string_from_log_parts(parts: Vec<LogPart>, level: Level) -> String {
        let time_str = helper::get_current_time_in_string();
        let date_str = helper::get_current_date_in_string();
        let mut res = String::new();
        for part in &parts {
            let temp = match part {
                LogPart::Time => &time_str,
                LogPart::Date => &date_str,
                LogPart::Level => &level.to_string(),
                LogPart::Text(tt) => tt,
                _ => {
                    eprintln!("Incrorrect part given!");
                    ""
                }
            };
            res.push_str(temp);
        }
        res
    }
    //pub(crate)
    pub fn from_file_formatter(
        format: FileFormatter,
        level: Level,
    ) -> Result<FileName, FileNameFromFileFormatterError> {
        let mut parts = format.format;
        if parts.is_empty() {
            return Err(FileNameFromFileFormatterError::NoFormatProvided);
        }

        let txt = match parts.last() {
            Some(LogPart::Text(tt)) => tt.clone(),
            _ => return Err(FileNameFromFileFormatterError::IncorrectLastPart),
        };
        if !txt.contains('.') {
            return Err(FileNameFromFileFormatterError::NoFileExtensionProvided);
        }

        // Split from the right to separate the extension from the file name
        let mut iter = txt.rsplitn(2, '.');
        let extension = iter.next().unwrap(); // Safe because we checked for '.'
        let file_name_without_ext = iter.next().unwrap_or("");

        // Check if the extension is acceptable
        if !FileName::is_acceptable_file_extension(extension) {
            return Err(FileNameFromFileFormatterError::IncorrectFileExtension);
        }

        // Replace the last element with the file name part without the extension
        let parts_len = parts.len();
        parts[parts_len - 1] = LogPart::Text(file_name_without_ext.to_string());

        // Build the final file name
        let file_name = FileName::get_string_from_log_parts(parts, level);
        Ok(FileName {
            file_name,
            file_num: None,
            file_extension: extension.to_string(),
        })
    }
}
impl From<FileName> for String {
    fn from(value: FileName) -> Self {
        let mut txt = value.file_name;
        match value.file_num {
            Some(num) => txt.push_str(&num.to_string()),
            None => {}
        };
        txt.push_str(".");
        txt.push_str(&value.file_extension);
        txt
    }
}
