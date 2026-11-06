use std::collections::{BTreeMap, HashSet};

use serde::Serialize;

use crate::{
    BarStats, ComputedLine, ComputedVisit, LocationId, Project, Severity, TravelTable, Violation,
    ViolationId, bar_gap_violations, bar_timelines, check_capacity, check_duplicate_bar,
    check_duration, check_incomplete_line, check_same_area, check_stay_times, check_stop_count,
    compute_all,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evaluation {
    pub lines: Vec<ComputedLine>,
    pub bars: BTreeMap<LocationId, Vec<ComputedVisit>>,
    pub stats: BTreeMap<LocationId, BarStats>,
    pub violations: Vec<Violation>,
}

/// Result of checking an edit: blocked if `after` has blocks that `before` didn't.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditCheck {
    pub allowed: bool,
    pub new_blocks: Vec<Violation>,
    pub evaluation: Evaluation,
}

/// Evaluates the project against all the rules
pub fn evaluate(project: &Project) -> Evaluation {
    let lines = compute_all(project);
    let bars = bar_timelines(&lines);
    let stats: BTreeMap<LocationId, BarStats> = bars
        .iter()
        .filter_map(|(id, visits)| {
            BarStats::from_visits(visits, project.settings.bar_gap_time).map(|stat| (*id, stat))
        })
        .collect();

    let travel_table = TravelTable::new(&project.travel_times);
    let mut violations = Vec::new();

    for (line, computed) in project.lines.iter().zip(&lines) {
        violations.extend(check_stop_count(line, &project.settings));
        violations.extend(check_duration(computed));
        violations.extend(check_duplicate_bar(line));
        violations.extend(check_stay_times(line, &project.settings));
        violations.extend(check_same_area(line, project, &travel_table));
        violations.extend(check_incomplete_line(computed));
    }
    violations.extend(check_capacity(project, &bars));
    violations.extend(bar_gap_violations(&bars, &project.settings));

    Evaluation {
        lines,
        bars,
        stats,
        violations,
    }
}

/// Checks if an edit from `before` to `after` adds new blocks. Blocks already in `before` don't stop it.
pub fn check_edit(before: &Project, after: &Project) -> EditCheck {
    let before_ev = evaluate(before);
    let after_ev = evaluate(after);

    let before_blocks: HashSet<ViolationId> = before_ev
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Block)
        .map(Violation::identity)
        .collect();

    let new_blocks: Vec<Violation> = after_ev
        .violations
        .iter()
        .filter(|v| v.severity == Severity::Block && !before_blocks.contains(&v.identity()))
        .cloned()
        .collect();

    let allowed = new_blocks.is_empty();

    EditCheck {
        allowed,
        new_blocks,
        evaluation: after_ev,
    }
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use chrono::NaiveDate;

    use super::*;
    use crate::{
        AreaId, Line, Location, LocationKind, Minutes, Severity, TravelTime, ViolationKind, Visit,
        load_project,
    };

    fn example() -> Project {
        load_project(include_str!("../../../../examples/2026.jvms.json")).unwrap()
    }

    #[test]
    fn example_has_every_line_and_stats_for_every_bar() {
        let project = example();

        let evaluation = evaluate(&project);

        assert_eq!(evaluation.lines.len(), project.lines.len());
        assert!(evaluation.stats.keys().eq(evaluation.bars.keys()));
    }

    // Run with `cargo test example_has_no_blocks -- --nocapture` to read the violations by eye
    #[test]
    fn example_has_no_blocks() {
        let evaluation = evaluate(&example());

        dbg!(&evaluation.violations);
        assert!(
            evaluation
                .violations
                .iter()
                .all(|v| v.severity < Severity::Block)
        );
    }

    #[test]
    fn line_rules_are_included() {
        let mut project = example();
        let line = &mut project.lines[0];
        line.visits[1].location_id = line.visits[0].location_id;
        let line_id = line.id;

        let evaluation = evaluate(&project);

        assert!(evaluation.violations.iter().any(|v| matches!(
            v.kind,
            ViolationKind::DuplicateBar { line_id: id, .. } if id == line_id
        )));
    }

    #[test]
    fn bar_rules_are_included() {
        let mut project = example();
        for location in &mut project.locations {
            if let LocationKind::Bar { capacity, .. } = &mut location.kind {
                *capacity = NonZeroU32::MIN;
            }
        }

        let evaluation = evaluate(&project);

        assert!(
            evaluation
                .violations
                .iter()
                .any(|v| matches!(v.kind, ViolationKind::TooManyLines { .. }))
        );
    }

    // Start → bar (capacity 1) → end, 10 minutes each way
    fn tiny_project() -> (Project, LocationId) {
        let date = NaiveDate::from_ymd_opt(2026, 5, 23).unwrap();
        let mut project = Project::new(String::from("Jyväsmetro"), date);
        let bar = Location::new(
            String::from("Hiisi"),
            LocationKind::Bar {
                capacity: NonZeroU32::MIN,
                area_id: AreaId::new(),
                overload: None,
            },
            String::from("Lutakonaukio 3, 40100 Jyväskylä"),
            None,
            None,
        );
        let bar_id = bar.id;
        project.locations.push(bar);
        for (from_id, to_id) in [
            (project.start_location_id, bar_id),
            (bar_id, project.end_location_id),
        ] {
            project.travel_times.push(TravelTime {
                from_id,
                to_id,
                minutes: Minutes::new(10),
                distance_km: None,
            });
        }
        (project, bar_id)
    }

    fn add_line(project: &mut Project, number: u32, bar: LocationId) {
        let mut line = Line::new(number, None, false, "16:00".parse().unwrap(), None);
        line.visits.push(Visit {
            location_id: bar,
            stay_time: Minutes::new(40),
        });
        project.lines.push(line);
    }

    #[test]
    fn new_overlap_at_capacity_one_bar_is_not_allowed() {
        let (mut before, bar) = tiny_project();
        add_line(&mut before, 1, bar);
        let mut after = before.clone();
        add_line(&mut after, 2, bar);

        let check = check_edit(&before, &after);

        assert!(!check.allowed);
        assert_eq!(check.new_blocks.len(), 2);
        assert!(
            check
                .new_blocks
                .iter()
                .all(|v| v.severity == Severity::Block
                    && matches!(v.kind, ViolationKind::TooManyLines { .. }))
        );
    }

    #[test]
    fn unrelated_stay_time_change_is_allowed() {
        let (mut before, bar) = tiny_project();
        add_line(&mut before, 1, bar);
        let mut after = before.clone();
        after.lines[0].visits[0].stay_time = Minutes::new(45);

        let check = check_edit(&before, &after);

        assert!(check.allowed);
        assert_eq!(check.new_blocks, []);
    }

    // The overlap changes from 40 to 45 minutes, but it's the same block as before
    #[test]
    fn existing_block_does_not_stop_an_edit() {
        let (mut before, bar) = tiny_project();
        add_line(&mut before, 1, bar);
        add_line(&mut before, 2, bar);
        let mut after = before.clone();
        after.lines[0].visits[0].stay_time = Minutes::new(45);
        after.lines[1].visits[0].stay_time = Minutes::new(45);

        let check = check_edit(&before, &after);

        assert!(check.allowed);
        assert_eq!(check.new_blocks, []);
    }

    #[test]
    fn evaluation_is_of_the_after_project() {
        let (mut before, bar) = tiny_project();
        add_line(&mut before, 1, bar);
        let mut after = before.clone();
        add_line(&mut after, 2, bar);

        let check = check_edit(&before, &after);

        assert_eq!(check.evaluation, evaluate(&after));
    }
}
