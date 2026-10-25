/// Formats minutes since midnight as `HH:MM`.
///
/// # Panics
///
/// Panics if `minutes` is 1440 or more, so over 24 hours.
pub fn format_time(minutes: u16) -> String {
    assert!(minutes < 1440, "minutes out of range :-( {minutes}");

    let hours = minutes / 60;
    let remaining_minutes = minutes - hours * 60;

    format!("{hours:02}:{remaining_minutes:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    //use proptest::prelude::*;
    //use std::result;

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
}
