use std::num::NonZeroU32;

use crate::{AreaId, LineId, LocationId, Minutes, Settings, TimeOfDay, TooManyLinesThresholds};

/// Struct for coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub kind: LocationKind,
    pub address: String,
    pub coords: Option<Coordinates>,
    pub notes: Option<String>,
}

impl Location {
    /// Creates the location with generated id
    pub fn new(
        name: String,
        kind: LocationKind,
        address: String,
        coords: Option<Coordinates>,
        notes: Option<String>,
    ) -> Self {
        Self {
            id: LocationId::new(),
            name,
            kind,
            address,
            coords,
            notes,
        }
    }
}

/// Struct of a [`Line`]
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub id: LineId,
    pub number: u32,
    pub label: Option<String>,
    pub is_vip: bool,
    pub departure: TimeOfDay,
    pub bus_size: Option<u32>,
    pub visits: Vec<Visit>,
}

impl Line {
    /// Creates a new line with given information + generated id and empty visit Vec.
    pub fn new(
        number: u32,
        label: Option<String>,
        is_vip: bool,
        departure: TimeOfDay,
        bus_size: Option<u32>,
    ) -> Self {
        Self {
            id: LineId::new(),
            number,
            label,
            is_vip,
            departure,
            bus_size,
            visits: Vec::new(),
        }
    }
}

/// Struct for a visit in a bar
#[derive(Debug, Clone, PartialEq)]
pub struct Visit {
    pub location_id: LocationId,
    pub stay_time: Minutes,
}

/// Struct for Areas (Kortepohja, Keskusta, Kuokkala etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    pub id: AreaId,
    pub name: String,
}

impl Area {
    pub fn new(name: String) -> Self {
        Self {
            id: AreaId::new(),
            name,
        }
    }
}

/// Struct containing information of distances between locations
#[derive(Debug, Clone, PartialEq)]
pub struct TravelTime {
    pub from_id: LocationId,
    pub to_id: LocationId,
    pub minutes: Minutes,
    pub distance_km: Option<f64>,
}

/// What kind of location is, start spot, event end spot or a bar
#[derive(Debug, Clone, PartialEq)]
pub enum LocationKind {
    Start,
    End,
    Bar {
        capacity: NonZeroU32,
        area_id: AreaId,
        overload: Option<TooManyLinesThresholds>,
    },
}

impl LocationKind {
    /// Returns locations capacity if it is a bar, otherwise it returns `None`
    pub fn capacity(&self) -> Option<NonZeroU32> {
        match self {
            Self::Bar { capacity, .. } => Some(*capacity),
            _ => None,
        }
    }

    /// Gets the overload settings from a location. Only Bar has them, others are None.
    /// If Bar returns None (= it is not set yet), it returns the default values.
    pub fn overload_settings(&self, settings: &Settings) -> Option<TooManyLinesThresholds> {
        match self {
            Self::Bar { overload, .. } => Some(overload.unwrap_or(settings.bar_thresholds)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_line() -> Line {
        let mut test_line = Line::new(67, None, false, TimeOfDay::new(1200).unwrap(), Some(57));
        let loc_id1 = LocationId::new();
        let loc_id2 = LocationId::new();

        let visit1 = Visit {
            location_id: loc_id1,
            stay_time: Minutes::new(40),
        };
        let visit2 = Visit {
            location_id: loc_id2,
            stay_time: Minutes::new(27),
        };

        test_line.visits.push(visit1);
        test_line.visits.push(visit2);

        test_line
    }

    #[test]
    fn test_line_creation() {
        let result = create_line();
        assert_eq!(result.number, 67);
        assert_eq!(result.label, None);
        assert!(!result.is_vip);
        assert_eq!(result.departure, TimeOfDay::new(1200).unwrap());
        assert_eq!(result.bus_size, Some(57));
        assert_eq!(result.visits.len(), 2);
        assert_eq!(result.visits[0].stay_time.amount(), 40);
    }

    #[test]
    fn test_area_creation() {
        let result = Area::new(String::from("Kortepohja"));
        assert_eq!(result.name, String::from("Kortepohja"));
    }

    #[test]
    fn test_location_kind() {
        let result = LocationKind::Bar {
            capacity: NonZeroU32::new(3).unwrap(),
            area_id: AreaId::new(),
            overload: None,
        };

        assert_eq!(result.capacity(), NonZeroU32::new(3));
    }

    #[test]
    fn test_location_kind_start() {
        let result = LocationKind::Start;

        assert_eq!(result.capacity(), None);
    }

    #[test]
    fn test_location_kind_end() {
        let result = LocationKind::End;

        assert_eq!(result.capacity(), None);
    }

    #[test]
    fn test_location_kind_bar_overloads() {
        let result = LocationKind::Bar {
            capacity: NonZeroU32::new(3).unwrap(),
            area_id: AreaId::new(),
            overload: None,
        };
        let settings = Settings::default();

        assert_eq!(
            result.overload_settings(&settings).unwrap().minor,
            settings.bar_thresholds.minor
        );
    }

    #[test]
    fn test_location_kind_bar_overload_not_none() {
        let custom_settings = TooManyLinesThresholds {
            block: 67,
            ..TooManyLinesThresholds::default()
        };

        let result = LocationKind::Bar {
            capacity: NonZeroU32::new(3).unwrap(),
            area_id: AreaId::new(),
            overload: Some(custom_settings),
        };
        let settings = Settings::default();

        assert_eq!(result.overload_settings(&settings).unwrap().block, 67);
    }

    #[test]
    fn test_location_kind_start_overload() {
        let result = LocationKind::Start;
        let settings = Settings::default();

        assert_eq!(result.overload_settings(&settings), None);
    }
}
