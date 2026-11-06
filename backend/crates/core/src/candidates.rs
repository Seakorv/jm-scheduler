// I was tired so i let Claude do this part
use std::collections::HashSet;

use serde::Serialize;

use crate::{
    LineId, LocationId, LocationKind, Project, Severity, Violation, ViolationId, evaluate,
};

/// What choosing one bar for a visit slot would cause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub location_id: LocationId,
    /// Worst new violation, `None` if the bar causes nothing new.
    pub severity: Option<Severity>,
    /// Violations that only appear with this bar in the slot.
    pub violations: Vec<Violation>,
}

/// Tries every bar in one visit slot of a line and reports what each would cause.
/// Sorted from best to worst; bars with the same severity keep the project's order.
/// Returns `None` if the line or the slot doesn't exist.
pub fn candidates(
    project: &Project,
    line_id: LineId,
    visit_index: usize,
) -> Option<Vec<Candidate>> {
    let line_pos = project.lines.iter().position(|line| line.id == line_id)?;
    project.lines[line_pos].visits.get(visit_index)?;

    let existing: HashSet<ViolationId> = evaluate(project)
        .violations
        .iter()
        .map(Violation::identity)
        .collect();

    let mut candidates: Vec<Candidate> = project
        .locations
        .iter()
        .filter(|location| matches!(location.kind, LocationKind::Bar { .. }))
        .map(|bar| {
            let mut trial = project.clone();
            trial.lines[line_pos].visits[visit_index].location_id = bar.id;

            let violations: Vec<Violation> = evaluate(&trial)
                .violations
                .into_iter()
                .filter(|v| !existing.contains(&v.identity()))
                .collect();
            let severity = violations.iter().map(|v| v.severity).max();

            Candidate {
                location_id: bar.id,
                severity,
                violations,
            }
        })
        .collect();

    // Stable sort, and `None` sorts before `Some`, so clean bars come first.
    candidates.sort_by_key(|candidate| candidate.severity);
    Some(candidates)
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::NaiveDate;

    use super::*;
    use crate::{AreaId, Line, Location, Minutes, TravelTime, ViolationKind, Visit};

    fn bar(name: &str) -> Location {
        Location::new(
            String::from(name),
            LocationKind::Bar {
                capacity: NonZeroU32::new(3).unwrap(),
                area_id: AreaId::new(),
                overload: None,
            },
            String::new(),
            None,
            None,
        )
    }

    /// Bars a, b, c with travel times between everything, d with none.
    /// One line visits a then b.
    fn project() -> (Project, [LocationId; 4]) {
        let date = NaiveDate::from_ymd_opt(2026, 5, 23).unwrap();
        let mut project = Project::new(String::from("Jyväsmetro"), date);
        let bars = [bar("A"), bar("B"), bar("C"), bar("D")];
        let ids = bars.each_ref().map(|b| b.id);
        project.locations.extend(bars);

        let connected = [
            project.start_location_id,
            ids[0],
            ids[1],
            ids[2],
            project.end_location_id,
        ];
        for (i, &from_id) in connected.iter().enumerate() {
            for &to_id in &connected[i + 1..] {
                project.travel_times.push(TravelTime {
                    from_id,
                    to_id,
                    minutes: Minutes::new(10),
                    distance_km: None,
                });
            }
        }

        let mut line = Line::new(1, None, false, "16:00".parse().unwrap(), None);
        for location_id in [ids[0], ids[1]] {
            line.visits.push(Visit {
                location_id,
                stay_time: Minutes::new(40),
            });
        }
        project.lines.push(line);

        (project, ids)
    }

    #[test]
    fn missing_line_or_slot_gives_none() {
        let (project, _) = project();
        let line_id = project.lines[0].id;

        assert_eq!(candidates(&project, LineId::new(), 0), None);
        assert_eq!(candidates(&project, line_id, 2), None);
    }

    #[test]
    fn every_bar_is_a_candidate_sorted_best_first() {
        let (project, [a, b, c, d]) = project();

        let result = candidates(&project, project.lines[0].id, 0).unwrap();

        let order: Vec<_> = result.iter().map(|c| (c.location_id, c.severity)).collect();
        assert_eq!(
            order,
            [
                (a, None),
                (c, None),
                (d, Some(Severity::Warning)),
                (b, Some(Severity::Block)),
            ]
        );
    }

    #[test]
    fn current_bar_causes_nothing_new() {
        let (project, [a, ..]) = project();

        let result = candidates(&project, project.lines[0].id, 0).unwrap();
        let current = result.iter().find(|c| c.location_id == a).unwrap();

        assert_eq!(current.violations, []);
    }

    #[test]
    fn bar_already_on_the_line_reports_duplicate() {
        let (project, [_, b, ..]) = project();

        let result = candidates(&project, project.lines[0].id, 0).unwrap();
        let duplicate = result.iter().find(|c| c.location_id == b).unwrap();

        assert!(
            duplicate
                .violations
                .iter()
                .any(|v| matches!(v.kind, ViolationKind::DuplicateBar { .. }))
        );
    }
}
