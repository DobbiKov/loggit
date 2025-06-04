use crate::logger::formatter::LogPart;

use thiserror::Error;

#[derive(Clone, Debug)]
pub(crate) struct FileFormatter {
    pub(crate) format: Vec<LogPart>,
}

#[derive(Debug, Error)]
pub enum FileFormatterTryFromStringError {
    #[error("an incorrect character given: {0}")]
    IncorrectCharacterGiven(char),
    #[error("An empty string was provided!")]
    EmptyStringGiven,
    #[error("No file extension provided")]
    NoFileExtensionProvided,
    #[error("An incrorrect part was provided")]
    IncorrectFormatPartGiven,
}

impl FileFormatter {
    pub(crate) fn is_part_authorized(part: &LogPart) -> bool {
        !matches!(part, LogPart::Message | LogPart::File | LogPart::Line)
    }
    fn forbidden_characters() -> [char; 4] {
        ['<', '>', '&', '%']
    }
    pub(crate) fn try_from_string(
        format: &str,
    ) -> Result<FileFormatter, FileFormatterTryFromStringError> {
        for ch in FileFormatter::forbidden_characters() {
            if format.contains(ch) {
                return Err(FileFormatterTryFromStringError::IncorrectCharacterGiven(ch));
            }
        }
        let elems = match crate::logger::formatter::parse_string_to_logparts(format) {
            Ok(r) => r,
            Err(_) => return Err(FileFormatterTryFromStringError::IncorrectFormatPartGiven),
        };
        let last_elem = match elems.last() {
            None => return Err(FileFormatterTryFromStringError::EmptyStringGiven),
            Some(el) => el,
        };
        let text = match &last_elem {
            crate::logger::formatter::LogPart::Text(t) => t,
            _ => return Err(FileFormatterTryFromStringError::NoFileExtensionProvided),
        };
        if !text.contains(".") {
            return Err(FileFormatterTryFromStringError::NoFileExtensionProvided);
        }
        if text.len() == 1 || text.ends_with(".") {
            return Err(FileFormatterTryFromStringError::NoFileExtensionProvided);
        }
        for elem in &elems {
            if !FileFormatter::is_part_authorized(elem) {
                return Err(FileFormatterTryFromStringError::IncorrectFormatPartGiven);
            }
        }
        Ok(FileFormatter { format: elems })
    }
}
