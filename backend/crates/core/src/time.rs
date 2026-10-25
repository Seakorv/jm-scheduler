use crate::time::ParseTimeError::BadFormat;

const DAY_IN_MINUTES: u16 = 1440;

/// Errors returned by [`parse_time`].
#[derive(Debug, PartialEq)]
pub enum ParseTimeError {
    Empty,
    BadFormat,
    HoursOutOfRange,
    MinutesOutOfRange,
}

/// Errors for [`TimeOfDay`]
#[derive(Debug, PartialEq)]
pub enum ValidateTimeError {
    OverWholeDay,
}

/// Formats minutes since midnight as `HH:MM`.
///
/// # Examples
///
/// ```
/// use jmetro_core::format_time;
/// let minutes = 123;
/// let minutes_in_hhmm = format_time(123);
///
/// assert_eq!("02:03", minutes_in_hhmm);
/// ```
///
/// # Panics
///
/// Panics if `minutes` is 1440 or more, so over 24 hours.
pub fn format_time(minutes: u16) -> String {
    assert!(
        minutes < DAY_IN_MINUTES,
        "minutes out of range :-( {minutes}"
    );

    let hours = minutes / 60;
    let remaining_minutes = minutes - hours * 60;

    format!("{hours:02}:{remaining_minutes:02}")
}

/// Parses `HH:MM` string to that time in minutes in a Result
///
/// # Examples
///
/// ```
/// use jmetro_core::parse_time;
/// let hhmm = "17:20";
/// let time_as_minutes = parse_time(hhmm);
/// assert_eq!(Ok(1040), time_as_minutes)
/// ```
///
/// # Errors
///
/// - Incorrect formatting (17:55:12 etc.) returns [`ParseTimeError::BadFormat`]
/// - Minutes over or equal 60 return [`ParseTimeError::MinutesOutOfRange`]
/// - Hours over or equal 24 return [`ParseTimeError::HoursOutOfRange`]
/// - Empty string returns [`ParseTimeError::Empty`]
pub fn parse_time(s: &str) -> Result<u16, ParseTimeError> {
    if s.is_empty() {
        return Err(ParseTimeError::Empty);
    }

    let (hours, minutes) = s.split_once([':', '.']).ok_or(BadFormat)?;

    if minutes.len() != 2 {
        return Err(ParseTimeError::BadFormat);
    }

    let hours_int_result = hours.parse::<u16>();
    let minutes_int_result = minutes.parse::<u16>();

    let hours_int = hours_int_result.map_err(|_| ParseTimeError::BadFormat)?;
    let minutes_int = minutes_int_result.map_err(|_| ParseTimeError::BadFormat)?;

    if hours_int >= 24 {
        return Err(ParseTimeError::HoursOutOfRange);
    }
    if minutes_int >= 60 {
        return Err(ParseTimeError::MinutesOutOfRange);
    }

    let time_in_minutes = hours_int * 60 + minutes_int;

    Ok(time_in_minutes)
}

/// Time struct which validates that the given minutes are less than 24 hours
///
/// # Example
///
/// ```
/// use jmetro_core::TimeOfDay;
/// let midday = TimeOfDay::new(720).unwrap();
/// assert_eq!(midday.minutes(), 720);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeOfDay(u16);

impl TimeOfDay {
    /// Creates the struct and checks if the given minutes are within one day
    ///
    /// # Errors
    ///
    /// - Over or equal to 24 hours returns [`ValidateTimeError::OverWholeDay`]
    pub fn new(minutes: u16) -> Result<Self, ValidateTimeError> {
        if minutes >= DAY_IN_MINUTES {
            return Err(ValidateTimeError::OverWholeDay);
        }
        Ok(Self(minutes))
    }
    /// Returns the struct's time as minutes.
    pub fn minutes(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    #[test]
    fn zero_minutes() {
        let result = format_time(0);
        assert_eq!(result, "00:00");
    }

    #[test]
    fn one_and_half_hours() {
        let result = format_time(90);
        assert_eq!(result, "01:30");
    }

    #[test]
    fn second_line_departure_time() {
        let result = format_time(970);
        assert_eq!(result, "16:10");
    }

    #[test]
    fn almost_midnight() {
        let result = format_time(1439);
        assert_eq!(result, "23:59");
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn panics_at_1440() {
        format_time(1440);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn panics_at_over_a_day() {
        format_time(6767);
    }

    #[test]
    fn parses_correctly() {
        let result = parse_time("17:50");
        assert_eq!(result, Ok(1070));
    }

    #[test]
    fn midnight() {
        let result = parse_time("00:00");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn only_minutes() {
        let result = parse_time("00:57");
        assert_eq!(result, Ok(57));
    }

    #[test]
    fn only_hours() {
        let result = parse_time("19:00");
        assert_eq!(result, Ok(1140));
    }

    #[test]
    fn hours_out_of_range() {
        let result = parse_time("25:12");
        assert_eq!(result, Err(ParseTimeError::HoursOutOfRange));
    }

    #[test]
    fn minutes_out_of_range() {
        let result = parse_time("23:67");
        assert_eq!(result, Err(ParseTimeError::MinutesOutOfRange));
    }

    #[test]
    fn empty_string() {
        let result = parse_time("");
        assert_eq!(result, Err(ParseTimeError::Empty));
    }

    #[test]
    fn bad_format_string() {
        let result = parse_time("1233123");
        assert_eq!(result, Err(ParseTimeError::BadFormat));
    }

    #[test]
    fn time_with_seconds() {
        let result = parse_time("16:00:00");
        assert_eq!(result, Err(ParseTimeError::BadFormat));
    }

    #[test]
    fn letters_bad_format() {
        let result = parse_time("yoloswag");
        assert_eq!(result, Err(ParseTimeError::BadFormat));
    }

    #[test]
    fn parsing_accepts_dot() {
        let result = parse_time("16.51");
        assert_eq!(result, Ok(1011));
    }

    #[test]
    fn parsing_accepts_single_digit_hour() {
        let result = parse_time("7.30");
        assert_eq!(result, Ok(450));
    }

    #[test]
    fn minutes_single_digit() {
        let result = parse_time("7:5");
        assert_eq!(result, Err(ParseTimeError::BadFormat));
    }

    #[test]
    fn struct_out_of_range() {
        let result = TimeOfDay::new(1440);
        assert_eq!(result, Err(ValidateTimeError::OverWholeDay));
    }

    #[test]
    fn struct_minutes_boundary() {
        let result = TimeOfDay::new(1439).unwrap();
        assert_eq!(result.minutes(), 1439);
    }

    proptest! {
        #[test]
        fn format_any_minute(minutes in 0u16..1440) {
            prop_assert!(parse_time(&format_time(minutes)) == Ok(minutes));
        }
    }
}
