use std::time::{SystemTime, UNIX_EPOCH};

use crate::Level;

use super::formatter::{LogFormatter, LogPart};

#[derive(Clone)]
pub(crate) struct FileFormatter {
    format: Vec<LogPart>,
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
    pub(crate) fn from_string(format: String) -> FileFormatter {
        if format.contains("<") || format.contains(">") {
            panic!("The '<' and '>' are not allowed for a file name!");
        }
        let elems = crate::logger::formatter::parse_string_to_logparts(format);
        let last_elem = match elems.last() {
            None => panic!("You passed incorrect string!"),
            Some(el) => el,
        };
        let text = match &last_elem {
            super::formatter::LogPart::Text(t) => t,
            _ => panic!("The file extension must be provided!"),
        };
        if !text.contains(".") {
            panic!("The file extension must be provided!");
        }
        if text.len() == 1 || text.ends_with(".") {
            panic!("incorrect extension provided!");
        }
        for elem in &elems {
            if !FileFormatter::is_part_authorized(&elem) {
                panic!("The file format is incorrect!");
            }
        }
        FileFormatter { format: elems }
    }
}
