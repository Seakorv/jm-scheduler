use chrono::{NaiveTime, Timelike, format::ParseErrorKind};
use std::{fmt, str};

const DAY_IN_MINUTES: u32 = 1440;

/// Errors returned when parsing a [`TimeOfDay`] from a string.
#[derive(Debug, PartialEq)]
pub enum ParseTimeError {
    Empty,
    BadFormat,
    OutOfRange,
}

/// Errors for [`TimeOfDay`]
#[derive(Debug, PartialEq)]
pub enum ValidateTimeError {
    OverWholeDay,
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
pub struct TimeOfDay(u32);

impl TimeOfDay {
    /// Creates the struct and checks if the given minutes are within one day
    ///
    /// # Errors
    ///
    /// - Over or equal to 24 hours returns [`ValidateTimeError::OverWholeDay`]
    pub fn new(minutes: u32) -> Result<Self, ValidateTimeError> {
        if minutes >= DAY_IN_MINUTES {
            return Err(ValidateTimeError::OverWholeDay);
        }
        Ok(Self(minutes))
    }
    /// Returns the struct's time as minutes.
    pub fn minutes(self) -> u32 {
        self.0
    }
}

/// Formats minutes since midnight as `HH:MM`.
///
/// # Examples
///
/// ```
/// use jmetro_core::TimeOfDay;
/// let midday = TimeOfDay::new(720).unwrap();
/// let minutes_in_hhmm = midday.to_string();
///
/// assert_eq!("12:00", minutes_in_hhmm);
/// ```
impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let minutes = self.minutes();
        let time = NaiveTime::from_hms_opt(minutes / 60, minutes % 60, 0).ok_or(fmt::Error)?;

        write!(f, "{}", time.format("%H:%M"))
    }
}

/// Parses `HH:MM` string to that time in minutes in a Result
/// "16:05", "7:50" and "01:5" are all valid.
///
/// # Examples
///
/// ```
/// use jmetro_core::TimeOfDay;
/// let hhmm = "17:20";
/// let time: TimeOfDay = hhmm.parse().unwrap();
/// assert_eq!(time.minutes(), 1040);
/// ```
///
/// # Errors
///
/// - Incorrect formatting (17:55:12 etc.) returns [`ParseTimeError::BadFormat`]
/// - Out of range ("25:67") returns [`ParseTimeError::OutOfRange`]
/// - Empty string returns [`ParseTimeError::Empty`]
impl str::FromStr for TimeOfDay {
    type Err = ParseTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseTimeError::Empty);
        }
        let s_norm = s.replace('.', ":");

        let time = NaiveTime::parse_from_str(&s_norm, "%H:%M").map_err(|e| match e.kind() {
            ParseErrorKind::OutOfRange => ParseTimeError::OutOfRange,
            _ => ParseTimeError::BadFormat,
        })?;

        let time_in_minutes = time.hour() * 60 + time.minute();

        TimeOfDay::new(time_in_minutes).map_err(|_| ParseTimeError::OutOfRange)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    fn parse(s: &str) -> Result<u32, ParseTimeError> {
        s.parse::<TimeOfDay>().map(TimeOfDay::minutes)
    }

    #[test]
    fn zero_minutes() {
        let result = TimeOfDay::new(0).unwrap().to_string();
        assert_eq!(result, "00:00");
    }

    #[test]
    fn one_and_half_hours() {
        let result = TimeOfDay::new(90).unwrap().to_string();
        assert_eq!(result, "01:30");
    }

    #[test]
    fn second_line_departure_time() {
        let result = TimeOfDay::new(970).unwrap().to_string();
        assert_eq!(result, "16:10");
    }

    #[test]
    fn almost_midnight() {
        let result = TimeOfDay::new(1439).unwrap().to_string();
        assert_eq!(result, "23:59");
    }

    #[test]
    fn struct_over_a_day() {
        let result = TimeOfDay::new(6767);
        assert_eq!(result, Err(ValidateTimeError::OverWholeDay));
    }

    #[test]
    fn parses_correctly() {
        let result = parse("17:50");
        assert_eq!(result, Ok(1070));
    }

    #[test]
    fn midnight() {
        let result = parse("00:00");
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn only_minutes() {
        let result = parse("00:57");
        assert_eq!(result, Ok(57));
    }

    #[test]
    fn only_hours() {
        let result = parse("19:00");
        assert_eq!(result, Ok(1140));
    }

    #[test]
    fn hours_out_of_range() {
        let result = parse("25:12");
        assert_eq!(result, Err(ParseTimeError::OutOfRange));
    }

    #[test]
    fn minutes_out_of_range() {
        let result = parse("23:67");
        assert_eq!(result, Err(ParseTimeError::OutOfRange));
    }

    #[test]
    fn empty_string() {
        let result = parse("");
        assert_eq!(result, Err(ParseTimeError::Empty));
    }

    #[test]
    fn bad_format_string() {
        let result = parse("1233123");
        assert_eq!(result, Err(ParseTimeError::BadFormat));
    }

    #[test]
    fn time_with_seconds() {
        let result = parse("16:00:00");
        assert_eq!(result, Err(ParseTimeError::BadFormat));
    }

    #[test]
    fn letters_bad_format() {
        let result = parse("yoloswag");
        assert_eq!(result, Err(ParseTimeError::BadFormat));
    }

    #[test]
    fn parsing_accepts_dot() {
        let result = parse("16.51");
        assert_eq!(result, Ok(1011));
    }

    #[test]
    fn parsing_accepts_single_digit_minutes() {
        let result = parse("07.3");
        assert_eq!(result, Ok(423));
    }

    #[test]
    fn parsing_accepts_single_digit_hour() {
        let result = parse("7.30");
        assert_eq!(result, Ok(450));
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
        fn format_any_minute(minutes in 0u32..1440) {
            let formatted = TimeOfDay::new(minutes).unwrap().to_string();
            prop_assert_eq!(parse(&formatted), Ok(minutes));
        }
    }
}
