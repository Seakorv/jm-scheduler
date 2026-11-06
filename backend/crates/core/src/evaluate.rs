use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    BarStats, ComputedLine, ComputedVisit, LocationId, Project, TravelTable, Violation,
    bar_gap_violations, bar_timelines, check_capacity, check_duplicate_bar, check_duration,
    check_incomplete_line, check_same_area, check_stay_times, check_stop_count, compute_all,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evaluation {
    pub lines: Vec<ComputedLine>,
    pub bars: BTreeMap<LocationId, Vec<ComputedVisit>>,
    pub stats: BTreeMap<LocationId, BarStats>,
    pub violations: Vec<Violation>,
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

// Claude generated tests
#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::{LocationKind, Severity, ViolationKind, load_project};

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
}
