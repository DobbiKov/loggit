use chrono::{self, Datelike, Timelike};
use std::{fmt::Display, io::Write};
use thiserror::Error;

pub(crate) fn get_current_time_in_utc() -> (u32, u32, i32, u32, u32, u32) {
    let date_time = chrono::Utc::now();
    let (day, month, year) = (
        date_time.date_naive().day(),
        date_time.date_naive().month(),
        date_time.date_naive().year(),
    );
    let (hour, minute, second) = (
        date_time.time().hour(),
        date_time.time().minute(),
        date_time.time().second(),
    );
    (day, month, year, hour, minute, second)
}

pub(crate) fn get_current_date_in_string() -> String {
    let (day, month, year, _, _, _) = get_current_time_in_utc();
    format!("{}-{}-{}", day, month, year)
}

pub(crate) fn get_current_time_in_string() -> String {
    let (_, _, _, hour, minute, second) = get_current_time_in_utc();
    format!("{}:{}:{}", hour, minute, second)
}

pub(crate) fn seconds_to_ymdhms(mut seconds: u64) -> (u64, u64, u64, u64, u64, u64) {
    const SECONDS_IN_MINUTE: u64 = 60;
    const SECONDS_IN_HOUR: u64 = 60 * SECONDS_IN_MINUTE;
    const SECONDS_IN_DAY: u64 = 24 * SECONDS_IN_HOUR;

    let mut year = 1970;
    let mut month = 1;
    let mut day = 1;

    let mut days = seconds / SECONDS_IN_DAY;
    seconds %= SECONDS_IN_DAY;

    let hour = seconds / SECONDS_IN_HOUR;
    seconds %= SECONDS_IN_HOUR;

    let minute = seconds / SECONDS_IN_MINUTE;
    let second = seconds % SECONDS_IN_MINUTE;

    let mut is_leap = |y: u64| -> bool { (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) };

    let days_in_month = |y: u64, m: u64| -> u64 {
        match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if is_leap(y) {
                    29
                } else {
                    28
                }
            }
            _ => {
                eprintln!("Invalid month given");
                0
            }
        }
    };

    // Calculate the year
    while days >= if is_leap(year) { 366 } else { 365 } {
        days -= if is_leap(year) { 366 } else { 365 };
        year += 1;
    }

    // Calculate the month
    while days >= days_in_month(year, month) {
        days -= days_in_month(year, month);
        month += 1;
    }

    day += days; // Remaining days count as the day of the month

    (year, month, day, hour, minute, second)
}

#[derive(Debug, Error)]
pub(crate) enum WriteToFileError {
    #[error("unexpected error")]
    UnexpectedError(std::io::Error),
}
pub(crate) fn write_to_file(file_name: &String, text: &String) -> Result<(), WriteToFileError> {
    let mut file = match std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name)
    {
        Ok(f) => f,
        Err(e) => {
            return Err(WriteToFileError::UnexpectedError(e));
        }
    };
    writeln!(file, "{}", text).map_err(WriteToFileError::UnexpectedError)
}
