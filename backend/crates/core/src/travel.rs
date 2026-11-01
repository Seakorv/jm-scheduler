use std::collections::HashMap;

use crate::{LocationId, Minutes, TravelTime};

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

    /// Returns the minutes between locations.
    pub fn raw(&self, from: LocationId, to: LocationId) -> Option<Minutes> {
        self.times.get(&(from, to)).copied()
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
}
