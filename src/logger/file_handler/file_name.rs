use crate::{helper, Level};

use crate::logger::formatter::LogPart;

use super::file_formatter::FileFormatter;
use thiserror::Error;
#[derive(Debug, Clone)]
pub(crate) struct FileName {
    file_name: String,
    file_num: Option<u32>,
    file_extension: String,
}

#[derive(Debug, Error)]
pub enum FileNameFromFileFormatterError {
    #[error("no fomrat provided")]
    NoFormatProvided,
    #[error("incorrect last part")]
    IncorrectLastPart,
    #[error("no file extension provided")]
    NoFileExtensionProvided,
    #[error("incorrect file extension")]
    IncorrectFileExtension,
}

impl FileName {
    fn acceptable_file_extensions() -> Vec<String> {
        vec!["txt", "log"]
            .into_iter()
            .map(|x| x.to_string())
            .collect()
    }
    fn is_acceptable_file_extension(ext: &str) -> bool {
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
    pub(crate) fn get_full_file_name(&self) -> String {
        String::from(self.to_owned())
    }
}
impl From<FileName> for String {
    fn from(value: FileName) -> Self {
        let mut txt = value.file_name;
        if let Some(num) = value.file_num {
            txt.push('(');
            txt.push_str(&num.to_string());
            txt.push(')');
        };
        txt.push('.');
        txt.push_str(&value.file_extension);
        txt
    }
}
