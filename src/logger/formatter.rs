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

impl From<String> for LogColor {
    fn from(value: String) -> Self {
        match value {
            val if val == *"red" => LogColor::Red,
            val if val == *"green" => LogColor::Green,
            val if val == *"blue" => LogColor::Blue,
            val if val == *"yellow" => LogColor::Yellow,
            val if val == *"black" => LogColor::Black,
            val if val == *"white" => LogColor::White,
            val if val == *"purple" => LogColor::Purple,
            _ => {
                eprintln!("Incorrect color given!");
                LogColor::White
            }
        }
    }
}

impl LogColor {
    fn get_colors_str() -> Vec<String> {
        vec!["red", "green", "blue", "yellow", "black", "white", "purple"]
            .into_iter()
            .map(|x| x.to_string())
            .collect()
    }
    pub(crate) fn get_ascii(&self) -> String {
        match self {
            LogColor::Red => "\x1b[38;2;255;0;0m",       // #FF0000
            LogColor::Green => "\x1b[38;2;0;255;0m",     // #00FF00
            LogColor::Blue => "\x1b[38;2;0;0;255m",      // #0000FF
            LogColor::Yellow => "\x1b[38;2;255;255;0m",  // #FFFF00
            LogColor::Black => "\x1b[38;2;0;0;0m",       // #000000
            LogColor::White => "\x1b[38;2;255;255;255m", // #FFFFFF
            LogColor::Purple => "\x1b[38;2;128;0;128m",  // #800080
        }
        .to_string()
    }

    pub(crate) fn colorize_str(text: &str, color: LogColor) -> String {
        format!("{}{}{}", color.get_ascii(), text, "\x1b[0m")
    }
}

#[derive(Debug, Clone)]
pub(crate) enum LogPart {
    Message,
    Time,
    File,
    Line,
    Date,
    Level,
    Text(String),
}

impl LogPart {
    fn get_parts_str() -> Vec<String> {
        vec![
            "message", "time", "date", "file", "line", "date", "level", "text",
        ]
        .into_iter()
        .map(|x| x.to_string())
        .collect()
    }
}

impl From<String> for LogPart {
    fn from(value: String) -> Self {
        match value {
            val if val == *"message" => LogPart::Message,
            val if val == *"time" => LogPart::Time,
            val if val == *"date" => LogPart::Date,
            val if val == *"file" => LogPart::File,
            val if val == *"line" => LogPart::Line,
            val if val == *"level" => LogPart::Level,
            _ => {
                eprintln!("Incorrect part given!");
                LogPart::Text(String::new())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LogFormatWrapper {
    pub(crate) color: Option<LogColor>,
    pub(crate) part: LogPart,
}

#[derive(Debug, Clone)]
pub(crate) struct LogFormatter {
    pub(crate) parts: Vec<LogFormatWrapper>,
}

impl LogFormatter {
    pub(crate) fn parse_from_string(text: &str) -> Self {
        let wrappers = parse_string_to_wrappers(text);
        LogFormatter { parts: wrappers }
    }
}
impl Default for LogFormatter {
    fn default() -> Self {
        LogFormatter::parse_from_string(
            "<green>[{level}]<green> <blue>({file} {line})<blue> - {message}",
        )
    }
}

/// Parse string to log_wrappers i.e Vec of log_part and assigned color to it
pub(crate) fn parse_string_to_wrappers(text: &str) -> Vec<LogFormatWrapper> {
    let symbols_struct = string_parse(text, "".to_string(), ParseSymbs::Start);
    let symbols = parse_symbs_to_vec(symbols_struct);
    let parts = parse_vec_of_parse_symb_to_parts(symbols).expect("Given string is incorrect!");
    parse_parts_to_formatter(parts).expect("Given string is incorrect!")
}

/// Parse string to log_parts
pub(crate) fn parse_string_to_logparts(text: &str) -> Vec<LogPart> {
    let wrappers = parse_string_to_wrappers(text);
    wrappers.into_iter().map(|x| x.part).collect()
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
            ParseParts::Color(text) => LogColor::get_colors_str().contains(text),
            ParseParts::BracketBlock(text) => LogPart::get_parts_str().contains(text),
            _ => true,
        }
    }
}

#[derive(Debug, Error)]
enum ParsePartsToFormatterError {
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
