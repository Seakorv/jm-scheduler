use std::collections::HashMap;

use crate::{LocationId, Minutes, TravelTime, round_up};

/// Struct which holds every travel time between locations
pub struct TravelTable {
    times: HashMap<(LocationId, LocationId), Minutes>,
}

impl TravelTable {
    /// Takes projects travel times and puts them into the `TravelTable` hashmap
    pub fn new(travel_times: &[TravelTime]) -> Self {
        let mut times: HashMap<(LocationId, LocationId), Minutes> = HashMap::new();

        for travel_time in travel_times {
            times.insert(
                (travel_time.from_id, travel_time.to_id),
                travel_time.minutes,
            );
        }

        Self { times }
    }

    /// Returns the non-rounded minutes between locations. If there is no travel time between A and B,
    /// it returns B to A, if it exists.
    pub fn raw(&self, from: LocationId, to: LocationId) -> Option<Minutes> {
        self.times
            .get(&(from, to))
            .or_else(|| self.times.get(&(to, from)))
            .copied()
    }

    /// Returns the rounded value of the travel time. Rounds up by the step.
    ///
    /// # Examples
    ///
    /// ```
    /// use jmetro_core::{LocationId, Minutes, TravelTable, TravelTime};
    ///
    /// let agora = LocationId::new();
    /// let bar = LocationId::new();
    /// let table = TravelTable::new(&[TravelTime {
    ///     from_id: agora,
    ///     to_id: bar,
    ///     minutes: Minutes::new(11),
    ///     distance_km: None,
    /// }]);
    ///
    /// assert_eq!(table.rounded(agora, bar, Minutes::new(5)), Some(Minutes::new(15)));
    /// ```
    pub fn rounded(&self, from: LocationId, to: LocationId, step: Minutes) -> Option<Minutes> {
        let raw_time = self.raw(from, to)?;
        let round_time = round_up(raw_time.amount(), step.amount());
        Some(Minutes::new(round_time))
    }
}

/// These tests are Claude-generated, important but boring to write :D
#[cfg(test)]
mod tests {
    use super::*;

    fn travel_time(from_id: LocationId, to_id: LocationId, minutes: u32) -> TravelTime {
        TravelTime {
            from_id,
            to_id,
            minutes: Minutes::new(minutes),
            distance_km: None,
        }
    }

    #[test]
    fn raw_finds_both_travel_times() {
        let a = LocationId::new();
        let b = LocationId::new();
        let c = LocationId::new();

        let table = TravelTable::new(&[travel_time(a, b, 10), travel_time(b, c, 15)]);

        assert_eq!(table.raw(a, b), Some(Minutes::new(10)));
        assert_eq!(table.raw(b, c), Some(Minutes::new(15)));
    }

    #[test]
    fn raw_returns_none_for_missing_pair() {
        let a = LocationId::new();
        let b = LocationId::new();
        let c = LocationId::new();

        let table = TravelTable::new(&[travel_time(a, b, 10)]);

        assert_eq!(table.raw(a, c), None);
    }

    #[test]
    fn raw_falls_back_to_reverse_direction() {
        let a = LocationId::new();
        let b = LocationId::new();

        let table = TravelTable::new(&[travel_time(a, b, 10)]);

        assert_eq!(table.raw(b, a), Some(Minutes::new(10)));
    }

    #[test]
    fn raw_prefers_explicit_direction() {
        let a = LocationId::new();
        let b = LocationId::new();

        let table = TravelTable::new(&[travel_time(a, b, 10), travel_time(b, a, 20)]);

        assert_eq!(table.raw(a, b), Some(Minutes::new(10)));
        assert_eq!(table.raw(b, a), Some(Minutes::new(20)));
    }

    #[test]
    fn rounded_rounds_up_to_step() {
        let a = LocationId::new();
        let b = LocationId::new();

        let table = TravelTable::new(&[travel_time(a, b, 11)]);

        assert_eq!(table.rounded(a, b, Minutes::new(5)), Some(Minutes::new(15)));
        assert_eq!(
            table.rounded(a, b, Minutes::new(10)),
            Some(Minutes::new(20))
        );
    }

    #[test]
    fn rounded_returns_none_for_missing_pair() {
        let a = LocationId::new();
        let b = LocationId::new();

        let table = TravelTable::new(&[]);

        assert_eq!(table.rounded(a, b, Minutes::new(5)), None);
    }
}
