use crate::{LocationKind, Project};
use std::collections::HashSet;
use std::hash::Hash;
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum ValidationIssue {
    #[error("invalid field: {0}")]
    Field(String),
    #[error("start location is missing or not a start")]
    BadStart,
    #[error("end location is missing or not an end")]
    BadEnd,
    #[error("visit is not a bar")]
    VisitNoBar,
    #[error("location's area is unknown")]
    UnknownArea,
    #[error("the location is unknown")]
    UnknownLocation,
    #[error("two locations have the same ID")]
    DuplicateLocationId,
    #[error("two areas have the same ID")]
    DuplicateAreaId,
    #[error("two lines have the same ID")]
    DuplicateLineId,
    #[error("two lines have the same number")]
    DuplicateLineNumber,
}

pub fn validate_project(project: &Project) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    check_start_and_end(project, &mut issues);
    check_visits(project, &mut issues);
    check_areas(project, &mut issues);
    check_travel_times(project, &mut issues);
    check_text_fields(project, &mut issues);
    check_settings(project, &mut issues);
    check_duplicates(project, &mut issues);
    issues
}

fn check_start_and_end(project: &Project, issues: &mut Vec<ValidationIssue>) {
    let start = project.location(project.start_location_id);
    let end = project.location(project.end_location_id);
    if !matches!(start, Some(location) if matches!(location.kind, LocationKind::Start)) {
        issues.push(ValidationIssue::BadStart);
    }
    if !matches!(end, Some(location) if matches!(location.kind, LocationKind::End)) {
        issues.push(ValidationIssue::BadEnd);
    }
}

fn check_visits(project: &Project, issues: &mut Vec<ValidationIssue>) {
    for line in &project.lines {
        for visit in &line.visits {
            let location = project.location(visit.location_id);

            if !matches!(location, Some(loc) if matches!(loc.kind, LocationKind::Bar { .. })) {
                issues.push(ValidationIssue::VisitNoBar);
            }
        }
    }
}

fn check_areas(project: &Project, issues: &mut Vec<ValidationIssue>) {
    for bar in project.bars() {
        if let LocationKind::Bar { area_id, .. } = bar.kind {
            let exists = project.areas.iter().any(|area| area.id == area_id);

            if !exists {
                issues.push(ValidationIssue::UnknownArea);
            }
        }
    }
}

fn check_travel_times(project: &Project, issues: &mut Vec<ValidationIssue>) {
    for travel_time in &project.travel_times {
        if project.location(travel_time.from_id).is_none() {
            issues.push(ValidationIssue::UnknownLocation);
        }
        if project.location(travel_time.to_id).is_none() {
            issues.push(ValidationIssue::UnknownLocation);
        }
    }
}

fn check_text_fields(project: &Project, issues: &mut Vec<ValidationIssue>) {
    if project.name.trim().is_empty() {
        issues.push(ValidationIssue::Field(String::from(
            "project name is empty",
        )));
    }

    for location in &project.locations {
        if location.name.trim().is_empty() {
            issues.push(ValidationIssue::Field(String::from(
                "location name is empty",
            )));
        }
        if location.address.trim().is_empty() {
            issues.push(ValidationIssue::Field(format!(
                "address of {} is empty",
                location.name
            )));
        }
    }

    for area in &project.areas {
        if area.name.trim().is_empty() {
            issues.push(ValidationIssue::Field(String::from("area name is empty")));
        }
    }
}

fn check_settings(project: &Project, issues: &mut Vec<ValidationIssue>) {
    if project.settings.stay_min_soft > project.settings.stay_max_soft {
        issues.push(ValidationIssue::Field(String::from(
            "minimum stay is longer than maximum stay",
        )));
    }
}

fn check_duplicates(project: &Project, issues: &mut Vec<ValidationIssue>) {
    if has_duplicates(project.locations.iter().map(|location| location.id)) {
        issues.push(ValidationIssue::DuplicateLocationId);
    }
    if has_duplicates(project.areas.iter().map(|area| area.id)) {
        issues.push(ValidationIssue::DuplicateAreaId);
    }
    if has_duplicates(project.lines.iter().map(|line| line.id)) {
        issues.push(ValidationIssue::DuplicateLineId);
    }
    if has_duplicates(project.lines.iter().map(|line| line.number)) {
        issues.push(ValidationIssue::DuplicateLineNumber);
    }
}

fn has_duplicates<T: Eq + Hash>(mut values: impl Iterator<Item = T>) -> bool {
    let mut seen = HashSet::new();
    values.any(|value| !seen.insert(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Area, AreaId, Line, Location, LocationId, Minutes, TimeOfDay, TravelTime, Visit};
    use chrono::NaiveDate;
    use std::num::NonZeroU32;

    fn bar(name: &str, area_id: AreaId) -> Location {
        Location::new(
            String::from(name),
            LocationKind::Bar {
                capacity: NonZeroU32::new(1).unwrap(),
                area_id,
                overload: None,
            },
            String::from("Kauppakatu 32, 40100 Jyväskylä"),
            None,
            None,
        )
    }

    fn line_visiting(location_id: LocationId) -> Line {
        let mut line = Line::new(1, None, false, TimeOfDay::new(960).unwrap(), None);
        line.visits.push(Visit {
            location_id,
            stay_time: Minutes::new(40),
        });
        line
    }

    /// A valid project with one area, one bar, one line and one travel time.
    fn valid_project() -> Project {
        let date = NaiveDate::from_ymd_opt(2026, 11, 14).unwrap();
        let mut project = Project::new(String::from("Jyväsmetro"), date);

        let area = Area::new(String::from("Keskusta"));
        let teerenpeli = bar("Teerenpeli", area.id);

        project.travel_times.push(TravelTime {
            from_id: project.start_location_id,
            to_id: teerenpeli.id,
            minutes: Minutes::new(10),
            distance_km: None,
        });
        project.lines.push(line_visiting(teerenpeli.id));
        project.areas.push(area);
        project.locations.push(teerenpeli);

        project
    }

    fn has_field_issue(project: &Project) -> bool {
        validate_project(project)
            .iter()
            .any(|issue| matches!(issue, ValidationIssue::Field(_)))
    }

    #[test]
    fn valid_project_has_no_issues() {
        assert_eq!(validate_project(&valid_project()), vec![]);
    }

    #[test]
    fn start_id_pointing_to_wrong_kind() {
        let mut project = valid_project();
        project.start_location_id = project.end_location_id;

        assert_eq!(validate_project(&project), vec![ValidationIssue::BadStart]);
    }

    #[test]
    fn unknown_end_id() {
        let mut project = valid_project();
        project.end_location_id = LocationId::new();

        assert_eq!(validate_project(&project), vec![ValidationIssue::BadEnd]);
    }

    #[test]
    fn visit_to_a_non_bar() {
        let mut project = valid_project();
        let mut line = line_visiting(project.start_location_id);
        line.number = 2;
        project.lines.push(line);

        assert_eq!(
            validate_project(&project),
            vec![ValidationIssue::VisitNoBar]
        );
    }

    #[test]
    fn bar_with_unknown_area() {
        let mut project = valid_project();
        project.locations.push(bar("Hiisi", AreaId::new()));

        assert_eq!(
            validate_project(&project),
            vec![ValidationIssue::UnknownArea]
        );
    }

    #[test]
    fn travel_time_with_unknown_location() {
        let mut project = valid_project();
        project.travel_times.push(TravelTime {
            from_id: project.start_location_id,
            to_id: LocationId::new(),
            minutes: Minutes::new(10),
            distance_km: None,
        });

        assert_eq!(
            validate_project(&project),
            vec![ValidationIssue::UnknownLocation]
        );
    }

    #[test]
    fn blank_project_name() {
        let mut project = valid_project();
        project.name = String::from("   ");

        assert!(has_field_issue(&project));
    }

    #[test]
    fn minimum_stay_longer_than_maximum() {
        let mut project = valid_project();
        project.settings.stay_min_soft = Minutes::new(60);

        assert!(has_field_issue(&project));
    }

    #[test]
    fn duplicate_location_id() {
        let mut project = valid_project();
        project.locations.push(project.locations[2].clone());

        assert_eq!(
            validate_project(&project),
            vec![ValidationIssue::DuplicateLocationId]
        );
    }

    #[test]
    fn duplicate_area_id() {
        let mut project = valid_project();
        project.areas.push(project.areas[0].clone());

        assert_eq!(
            validate_project(&project),
            vec![ValidationIssue::DuplicateAreaId]
        );
    }

    #[test]
    fn duplicate_line_id() {
        let mut project = valid_project();
        let mut line = project.lines[0].clone();
        line.number = 2;
        project.lines.push(line);

        assert_eq!(
            validate_project(&project),
            vec![ValidationIssue::DuplicateLineId]
        );
    }

    #[test]
    fn duplicate_line_number() {
        let mut project = valid_project();
        project.lines.push(line_visiting(project.locations[2].id));

        assert_eq!(
            validate_project(&project),
            vec![ValidationIssue::DuplicateLineNumber]
        );
    }
}
