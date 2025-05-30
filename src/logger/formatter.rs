use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LogColor {
    Red,
    Green,
    Blue,
    Yellow,
    Black,
    White,
    Purple,
}

impl From<&str> for LogColor {
    fn from(value: &str) -> Self {
        match value {
            "red" => LogColor::Red,
            "green" => LogColor::Green,
            "blue" => LogColor::Blue,
            "yellow" => LogColor::Yellow,
            "black" => LogColor::Black,
            "white" => LogColor::White,
            "purple" => LogColor::Purple,
            _ => {
                eprintln!("Incorrect color given!");
                LogColor::White
            }
        }
    }
}
impl From<String> for LogColor {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl LogColor {
    fn get_colors_str() -> [&'static str; 7] {
        ["red", "green", "blue", "yellow", "black", "white", "purple"]
    }
    pub(crate) fn get_ascii(&self) -> &'static str {
        match self {
            LogColor::Red => "\x1b[38;2;255;0;0m",       // #FF0000
            LogColor::Green => "\x1b[38;2;0;255;0m",     // #00FF00
            LogColor::Blue => "\x1b[38;2;0;0;255m",      // #0000FF
            LogColor::Yellow => "\x1b[38;2;255;255;0m",  // #FFFF00
            LogColor::Black => "\x1b[38;2;0;0;0m",       // #000000
            LogColor::White => "\x1b[38;2;255;255;255m", // #FFFFFF
            LogColor::Purple => "\x1b[38;2;128;0;128m",  // #800080
        }
    }

    pub(crate) fn colorize_str(text: &str, color: LogColor) -> String {
        format!("{}{}{}", color.get_ascii(), text, "\x1b[0m")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LogPart {
    Message,
    Time,
    File,
    Line,
    Date,
    Level,
    Text(String),
    ModulePath,
}

impl LogPart {
    fn get_parts_str() -> [&'static str; 9] {
        [
            "message", "time", "date", "file", "line", "date", "level", "text", "module",
        ]
    }
}

impl From<&str> for LogPart {
    fn from(value: &str) -> Self {
        match value {
            "message" => LogPart::Message,
            "time" => LogPart::Time,
            "date" => LogPart::Date,
            "file" => LogPart::File,
            "line" => LogPart::Line,
            "level" => LogPart::Level,
            "module" => LogPart::ModulePath,
            _ => {
                eprintln!("Incorrect part given!");
                LogPart::Text(String::new())
            }
        }
    }
}
impl From<String> for LogPart {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LogFormatWrapper {
    pub(crate) color: Option<LogColor>,
    pub(crate) part: LogPart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LogFormatter {
    pub(crate) parts: Vec<LogFormatWrapper>,
}

impl LogFormatter {
    pub(crate) fn parse_from_string(text: &str) -> Result<Self, ParseStringToWrappersError> {
        let wrappers = parse_string_to_wrappers(text)?;
        Ok(LogFormatter { parts: wrappers })
    }
}
impl Default for LogFormatter {
    fn default() -> Self {
        LogFormatter::parse_from_string("{file}-{line} <green>[{level}]<green> - {message}")
            .unwrap()
    }
}

#[derive(Debug, Error)]
pub enum ParseStringToWrappersError {
    #[error("couldn't parse symbols to the parts: {0}")]
    UnableToParseSymbolsToParts(ParseSymbToPartsError),
    #[error("couldn't parse parts to the formatter: {0}")]
    UnableToParsePartsToFormatter(ParsePartsToFormatterError),
}

/// Parse string to log_wrappers i.e Vec of log_part and assigned color to it
pub(crate) fn parse_string_to_wrappers(
    text: &str,
) -> Result<Vec<LogFormatWrapper>, ParseStringToWrappersError> {
    let symbols_struct = string_parse(text, "".to_string(), ParseSymbs::Start);
    let symbols = parse_symbs_to_vec(symbols_struct);
    let parts = match parse_vec_of_parse_symb_to_parts(symbols) {
        Ok(r) => r,
        Err(e) => {
            return Err(ParseStringToWrappersError::UnableToParseSymbolsToParts(e));
        }
    };
    parse_parts_to_formatter(parts)
        .map_err(ParseStringToWrappersError::UnableToParsePartsToFormatter)
}

/// Parse string to log_parts
pub(crate) fn parse_string_to_logparts(
    text: &str,
) -> Result<Vec<LogPart>, ParseStringToWrappersError> {
    let wrappers = parse_string_to_wrappers(text)?;
    Ok(wrappers.into_iter().map(|x| x.part).collect())
}

// ******
// The fundamental parser logic

#[derive(Debug)]
pub enum ParseSymbs {
    Start,
    AndNext(Box<ParseSymbs>, Box<ParseSymbs>),
    AngleOpen,
    AngleClose,
    Text(String),
    BracketOpen,
    BracketClose,
}

impl From<ParseSymbs> for Vec<ParseSymbs> {
    fn from(val: ParseSymbs) -> Self {
        parse_symbs_to_vec(val)
    }
}

#[derive(Debug)]
pub enum ParseParts {
    End,
    Text(String),
    Color(String),
    BracketBlock(String),
}
impl ParseParts {
    //verify if the text in block and color is correct
    fn verify_color_block_integriy(&self) -> bool {
        match self {
            ParseParts::Color(text) => LogColor::get_colors_str().contains(&text.as_str()),
            ParseParts::BracketBlock(text) => LogPart::get_parts_str().contains(&text.as_str()),
            _ => true,
        }
    }
}

#[derive(Debug, Error)]
pub enum ParsePartsToFormatterError {
    #[error("unexpected error")]
    UnexpectedError,
    #[error("incorrect data given")]
    IncorrectDataGiven,
}

fn parse_parts_to_formatter(
    parts: Vec<ParseParts>,
) -> Result<Vec<LogFormatWrapper>, ParsePartsToFormatterError> {
    let mut res: Vec<LogFormatWrapper> = Vec::new();
    let mut curr_color: Option<LogColor> = None;

    let iterator = parts.into_iter();

    for el in iterator {
        match el {
            ParseParts::End => {}
            ParseParts::Text(text) => res.push(LogFormatWrapper {
                color: curr_color,
                part: LogPart::Text(text),
            }),
            ParseParts::Color(text) => match curr_color {
                None => curr_color = Some(text.into()), // if current color is none, then we open
                // color block and change it
                Some(color) => match color {
                    c if c == curr_color.unwrap() => {
                        // if current color equals to the
                        // block we read, then we close
                        // the color part and change color
                        // back to None
                        curr_color = None;
                    }
                    _ => return Err(ParsePartsToFormatterError::IncorrectDataGiven), // in other
                                                                                     // case,
                                                                                     // colors
                                                                                     // don't match
                                                                                     // so the
                                                                                     // blocks are
                                                                                     // incorrect
                },
            },
            ParseParts::BracketBlock(text) => res.push(LogFormatWrapper {
                color: curr_color,
                part: text.into(),
            }),
        }
    }
    if curr_color.is_some() {
        return Err(ParsePartsToFormatterError::IncorrectDataGiven);
    }

    Ok(res)
}

#[derive(Debug, Error)]
pub enum ParseSymbToPartsError {
    #[error("incorrect data given")]
    IncorrectDataGiven,
    #[error("unexpected error")]
    UnexpectedError,
}

fn parse_vec_of_parse_symb_to_parts(
    symbs: Vec<ParseSymbs>,
) -> Result<Vec<ParseParts>, ParseSymbToPartsError> {
    let mut res: Vec<ParseParts> = Vec::new();
    let mut iterator = symbs.into_iter();

    while let Some(el) = iterator.next() {
        match el {
            ParseSymbs::Start => {}
            ParseSymbs::AndNext(_, _) => return Err(ParseSymbToPartsError::UnexpectedError),
            ParseSymbs::Text(text) => res.push(ParseParts::Text(text)),
            ParseSymbs::AngleOpen => {
                let text_in_between = match iterator.next() {
                    Some(ParseSymbs::Text(text)) => text,
                    _ => return Err(ParseSymbToPartsError::IncorrectDataGiven),
                };
                match iterator.next() {
                    Some(ParseSymbs::AngleClose) => res.push(ParseParts::Color(text_in_between)),
                    _ => return Err(ParseSymbToPartsError::IncorrectDataGiven),
                }
            }
            ParseSymbs::BracketOpen => {
                let text_in_between = match iterator.next() {
                    Some(ParseSymbs::Text(text)) => text,
                    _ => return Err(ParseSymbToPartsError::IncorrectDataGiven),
                };
                match iterator.next() {
                    Some(ParseSymbs::BracketClose) => {
                        res.push(ParseParts::BracketBlock(text_in_between))
                    }
                    _ => return Err(ParseSymbToPartsError::IncorrectDataGiven),
                }
            }
            _ => return Err(ParseSymbToPartsError::IncorrectDataGiven),
        }
    }
    let temp: Vec<bool> = res
        .iter()
        .map(|x| x.verify_color_block_integriy())
        .collect();
    if temp.contains(&false) {
        return Err(ParseSymbToPartsError::IncorrectDataGiven);
    }
    Ok(res)
}

fn parse_symbs_to_vec(symbs: ParseSymbs) -> Vec<ParseSymbs> {
    let mut res = Vec::<ParseSymbs>::new();
    let mut curr = symbs;
    loop {
        match curr {
            ParseSymbs::AndNext(next, el) => {
                res.push(*el);
                curr = *next;
            }
            e => {
                res.push(e);
                break;
            }
        }
    }
    res.reverse();
    res
}

fn string_parse(string: &str, acc_text: String, acc1: ParseSymbs) -> ParseSymbs {
    if string.is_empty() {
        if !acc_text.is_empty() {
            ParseSymbs::AndNext(Box::new(acc1), Box::new(ParseSymbs::Text(acc_text)))
        } else {
            acc1
        }
    } else {
        let curr_char = string.chars().next().unwrap();
        let mut acc_to_ret: ParseSymbs = acc1;
        let mut str_to_ret: String = acc_text;
        if ['{', '}', '<', '>'].contains(&curr_char) {
            acc_to_ret =
                ParseSymbs::AndNext(Box::new(acc_to_ret), Box::new(ParseSymbs::Text(str_to_ret)));
            str_to_ret = String::new();
        }
        match curr_char {
            '{' => string_parse(
                &string[1..],
                str_to_ret,
                ParseSymbs::AndNext(Box::new(acc_to_ret), Box::new(ParseSymbs::BracketOpen)),
            ),
            '}' => string_parse(
                &string[1..],
                str_to_ret,
                ParseSymbs::AndNext(Box::new(acc_to_ret), Box::new(ParseSymbs::BracketClose)),
            ),
            '<' => string_parse(
                &string[1..],
                str_to_ret,
                ParseSymbs::AndNext(Box::new(acc_to_ret), Box::new(ParseSymbs::AngleOpen)),
            ),
            '>' => string_parse(
                &string[1..],
                str_to_ret,
                ParseSymbs::AndNext(Box::new(acc_to_ret), Box::new(ParseSymbs::AngleClose)),
            ),
            el => {
                str_to_ret.push(el);
                string_parse(&string[1..], str_to_ret, acc_to_ret)
            }
        }
    }
}
