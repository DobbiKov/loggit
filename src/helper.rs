use std::io::Write;

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
            _ => panic!("Invalid month"),
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

pub(crate) enum WriteToFileError {
    UnexpectedError,
}
pub(crate) fn write_to_file(file_name: &String, text: &String) -> Result<(), WriteToFileError> {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name)
        .unwrap();

    if let Err(e) = writeln!(file, "{}", text) {
        eprintln!("Couldn't write to file: {}", e);
        Err(WriteToFileError::UnexpectedError)
    } else {
        Ok(())
    }
    //
}
