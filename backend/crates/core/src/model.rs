use crate::{AreaId, LineId, LocationId, Minutes, TimeOfDay};

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
}
