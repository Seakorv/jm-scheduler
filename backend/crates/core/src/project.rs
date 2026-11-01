use chrono::NaiveDate;

use crate::{Area, Line, Location, LocationId, LocationKind, Settings, TravelTime};

pub const SCHEMA_VERSION: u32 = 1;

/// Struct for creating project. Schema version saves the version of this struct, so
/// updating doesn't allow old versions etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub schema_version: u32,
    pub name: String,
    pub event_date: NaiveDate,
    pub settings: Settings,
    pub areas: Vec<Area>,
    pub locations: Vec<Location>,
    pub start_location_id: LocationId,
    pub end_location_id: LocationId,
    pub travel_times: Vec<TravelTime>,
    pub lines: Vec<Line>,
}

impl Project {
    /// Sets up a project with defaults. User adds the name of the event and the date of the event.
    pub fn new(name: String, event_date: NaiveDate) -> Self {
        let start = Location::new(
            String::from("Agora"),
            LocationKind::Start,
            String::from("Mattilanniemi 2, 40100 Jyväskylä"),
            None,
            None,
        );

        let end = Location::new(
            String::from("Keskusta"),
            LocationKind::End,
            String::from("Urhonkatu 1, 40100 Jyväskylä"),
            None,
            None,
        );

        Self {
            schema_version: SCHEMA_VERSION,
            name,
            event_date,
            settings: Settings::default(),
            areas: Vec::new(),
            start_location_id: start.id,
            end_location_id: end.id,
            locations: vec![start, end],
            travel_times: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Finds a specific location from the project matching to the location id
    pub fn location(&self, id: LocationId) -> Option<&Location> {
        self.locations.iter().find(|location| id == location.id)
    }

    /// Returns an iterator of locations which are bars, so ignores starts and ends.
    pub fn bars(&self) -> impl Iterator<Item = &Location> {
        self.locations
            .iter()
            .filter(|location| matches!(location.kind, LocationKind::Bar { .. }))
    }
}

#[cfg(test)]
mod tests {

    use std::num::NonZeroU32;

    use crate::AreaId;

    use super::*;

    #[test]
    fn test_project_creation() {
        let start_date: NaiveDate = "2067-4-27".parse().unwrap();
        let result = Project::new(String::from("JM 2067"), start_date);

        assert_eq!(result.locations[0].id, result.start_location_id);
        assert_eq!(result.name, String::from("JM 2067"));
        assert_eq!(result.locations.len(), 2);
    }

    #[test]
    fn test_project_query_methods() {
        let start_date: NaiveDate = "2067-4-27".parse().unwrap();
        let mut project = Project::new(String::from("JM 67"), start_date);
        let start_location = project.location(project.start_location_id);

        assert_eq!(start_location.unwrap().name, String::from("Agora"));
        assert_eq!(project.bars().count(), 0);
        assert_eq!(project.location(LocationId::new()), None);

        let new_bar = Location::new(
            String::from("Testibaari"),
            LocationKind::Bar {
                capacity: NonZeroU32::new(1).unwrap(),
                area_id: AreaId::new(),
                overload: None,
            },
            String::from("Kissakuja 3"),
            None,
            None,
        );

        project.locations.push(new_bar);

        assert_eq!(project.bars().count(), 1);
    }
}
