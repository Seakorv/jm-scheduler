/// Rounds the provided number up to the next multiple of the step.
/// When step is zero, function returns the number, so basically it just passes through the first parameter.
///
/// # Panics
///
/// Function doesn't deal with unrealistic overflow of u16
/// `round_up(65534, 7)` for example. Real travel times are far less than that.
pub fn round_up(number: u16, step: u16) -> u16 {
    if step == 0 {
        return number;
    }

    let remainder = number % step;

    if remainder == 0 {
        number
    } else {
        let addition = step - remainder;
        number + addition
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn round_up_to_five() {
        let result = round_up(12, 5);
        assert_eq!(result, 15);
    }
    #[test]
    fn already_rounded() {
        let result = round_up(45, 5);
        assert_eq!(result, 45);
    }
    #[test]
    fn step_is_zero() {
        let result = round_up(7, 0);
        assert_eq!(result, 7);
    }
    #[test]
    fn step_is_one() {
        let result = round_up(7, 1);
        assert_eq!(result, 7);
    }
    #[test]
    fn step_is_seven() {
        let result = round_up(50, 7);
        assert_eq!(result, 56);
    }
    #[test]
    fn number_is_zero() {
        let result = round_up(0, 5);
        assert_eq!(result, 0);
    }
    #[test]
    fn step_is_ten() {
        let result = round_up(11, 10);
        assert_eq!(result, 20);
    }
    #[test]
    fn exact_multiple_at_step() {
        let result = round_up(67, 67);
        assert_eq!(result, 67);
    }
    #[test]
    fn number_smaller_than_step() {
        let result = round_up(1, 5);
        assert_eq!(result, 5);
    }

    // Realistic inputs: travel time is max 24 hours, steps max 60 minutes.
    proptest! {
        #[test]
        fn result_is_never_smaller(number in 0u16..=1440, step in 1u16..=60) {
            prop_assert!(round_up(number, step) >= number);
        }

        #[test]
        fn result_is_a_multiple_of_step(number in 0u16..=1440, step in 1u16..=60) {
            prop_assert_eq!(round_up(number, step) % step, 0);
        }

        #[test]
        fn result_is_less_than_one_step_above(number in 0u16..=1440, step in 1u16..=60) {
            prop_assert!(round_up(number, step) - number < step);
        }

        #[test]
        fn zero_step_returns_number_unchanged(number in any::<u16>()) {
            prop_assert_eq!(round_up(number, 0), number);
        }
    }
}
