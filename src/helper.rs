pub(crate) fn seconds_to_ymdhms(mut seconds: u64) -> (u64, u64, u64, u64, u64, u64) {
    const SECONDS_IN_MINUTE: u64 = 60;
    const SECONDS_IN_HOUR: u64 = 60 * SECONDS_IN_MINUTE;
    const SECONDS_IN_DAY: u64 = 24 * SECONDS_IN_HOUR;
    const SECONDS_IN_MONTH: u64 = 30 * SECONDS_IN_DAY; // Approximate month length
    const SECONDS_IN_YEAR: u64 = 365 * SECONDS_IN_DAY; // Non-leap year

    let years = seconds / SECONDS_IN_YEAR;
    seconds %= SECONDS_IN_YEAR;

    let months = seconds / SECONDS_IN_MONTH;
    seconds %= SECONDS_IN_MONTH;

    let days = seconds / SECONDS_IN_DAY;
    seconds %= SECONDS_IN_DAY;

    let hours = seconds / SECONDS_IN_HOUR;
    seconds %= SECONDS_IN_HOUR;

    let minutes = seconds / SECONDS_IN_MINUTE;
    seconds %= SECONDS_IN_MINUTE;

    (years, months, days, hours, minutes, seconds)
}
