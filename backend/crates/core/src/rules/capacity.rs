use std::collections::BTreeMap;

use crate::{
    ComputedVisit, LocationId, Minutes, Project, Segment, Severity, TooManyLinesThresholds,
    Violation, ViolationKind, bar_segments,
};

/// Checks the amount of overlaoded minutes in a bar.
fn overloaded_minutes(visit: &ComputedVisit, segments: &[Segment], capacity: u32) -> Minutes {
    Minutes::new(
        segments
            .iter()
            .filter(|s| s.is_within(visit) && s.count > capacity)
            .map(|s| s.end.minutes() - s.start.minutes())
            .sum(),
    )
}

/// The severity of overloads in > 1 capacity bars
fn ratio_severity(ratio_pct: u32, thresholds: &TooManyLinesThresholds) -> Option<Severity> {
    match ratio_pct {
        r if r > thresholds.block => Some(Severity::Block),
        r if r >= thresholds.warning => Some(Severity::Warning),
        r if r >= thresholds.minor => Some(Severity::Slight),
        _ => None,
    }
}

/// Returns the severity of overload minutes in one capacity bars. Slight warning on leeway, otherwise block
fn minutes_severity(overloaded: Minutes, thresholds: &TooManyLinesThresholds) -> Option<Severity> {
    match overloaded {
        o if o.amount() == 0 => None,
        o if o <= thresholds.simultaneous_leeway => Some(Severity::Slight),
        _ => Some(Severity::Block),
    }
}

/// Checks the bars capacity and returns violations if there are any
pub fn check_capacity(
    project: &Project,
    timelines: &BTreeMap<LocationId, Vec<ComputedVisit>>,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (location_id, visits) in timelines {
        let Some(bar) = project.location(*location_id) else {
            continue;
        };
        let (Some(capacity), Some(thresholds)) = (
            bar.kind.capacity(),
            bar.kind.overload_settings(&project.settings),
        ) else {
            continue;
        };
        let segments = bar_segments(visits);

        for visit in visits {
            let overloaded = overloaded_minutes(visit, &segments, capacity.get());
            let stay_time = visit.departure.minutes() - visit.arrival.minutes();
            let Some(ratio_pct) = (overloaded.amount() * 100).checked_div(stay_time) else {
                continue;
            };
            if capacity.get() == 1 {
                let Some(severity) = minutes_severity(overloaded, &thresholds) else {
                    continue;
                };
                violations.push(Violation {
                    severity,
                    kind: ViolationKind::TooManyLines {
                        line_id: visit.line_id,
                        location_id: visit.location_id,
                        visit_index: visit.index,
                        overloaded,
                        ratio_pct,
                    },
                });
            } else {
                let Some(severity) = ratio_severity(ratio_pct, &thresholds) else {
                    continue;
                };
                violations.push(Violation {
                    severity,
                    kind: ViolationKind::TooManyLines {
                        line_id: visit.line_id,
                        location_id: visit.location_id,
                        visit_index: visit.index,
                        overloaded,
                        ratio_pct,
                    },
                });
            }
        }
    }
    violations
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AreaId, LineId, Location, LocationId, LocationKind, TimeOfDay, bar_segments};
    use chrono::NaiveDate;
    use proptest::prelude::*;
    use std::num::NonZeroU32;

    fn time(s: &str) -> TimeOfDay {
        s.parse().unwrap()
    }

    fn visit(arrival: &str, departure: &str) -> ComputedVisit {
        ComputedVisit {
            line_id: LineId::new(),
            location_id: LocationId::new(),
            index: 0,
            arrival: time(arrival),
            departure: time(departure),
            travel_in: Minutes::new(10),
        }
    }

    // R6 example: 3 lines present 18:10–18:40, so 30 of the first visit's 40 minutes are overloaded
    fn r6_visits() -> [ComputedVisit; 4] {
        [
            visit("18:00", "18:40"),
            visit("18:10", "18:50"),
            visit("18:10", "18:50"),
            visit("19:00", "19:40"),
        ]
    }

    #[test]
    fn r6_example_has_30_overloaded_minutes() {
        let visits = r6_visits();
        let segments = bar_segments(&visits);

        assert_eq!(
            overloaded_minutes(&visits[0], &segments, 2),
            Minutes::new(30)
        );
    }

    #[test]
    fn nothing_overloaded_when_capacity_is_enough() {
        let visits = r6_visits();
        let segments = bar_segments(&visits);

        assert_eq!(
            overloaded_minutes(&visits[0], &segments, 3),
            Minutes::new(0)
        );
    }

    #[test]
    fn overload_outside_the_visit_does_not_count() {
        let visits = r6_visits();
        let segments = bar_segments(&visits);

        assert_eq!(
            overloaded_minutes(&visits[3], &segments, 2),
            Minutes::new(0)
        );
    }

    /// Slow but obvious: count the lines present at every single minute of the visit.
    fn naive_overloaded(visit: &ComputedVisit, all: &[ComputedVisit], capacity: u32) -> u32 {
        let mut overloaded = 0;
        for minute in visit.arrival.minutes()..visit.departure.minutes() {
            let present = all
                .iter()
                .filter(|other| {
                    other.arrival.minutes() <= minute && minute < other.departure.minutes()
                })
                .count();
            if present > capacity as usize {
                overloaded += 1;
            }
        }
        overloaded
    }

    proptest! {
        #[test]
        fn sweep_line_matches_minute_by_minute_count(
            stays in prop::collection::vec((960u32..1200, 0u32..=60), 1..=8),
            capacity in 1u32..=3,
        ) {
            let visits: Vec<ComputedVisit> = stays
                .iter()
                .map(|&(start, length)| ComputedVisit {
                    line_id: LineId::new(),
                    location_id: LocationId::new(),
                    index: 0,
                    arrival: TimeOfDay::new(start).unwrap(),
                    departure: TimeOfDay::new(start + length).unwrap(),
                    travel_in: Minutes::new(10),
                })
                .collect();
            let segments = bar_segments(&visits);

            for visit in &visits {
                prop_assert_eq!(
                    overloaded_minutes(visit, &segments, capacity).amount(),
                    naive_overloaded(visit, &visits, capacity)
                );
            }
        }
    }

    #[test]
    fn ratio_severity_borders_follow_r6() {
        // default thresholds: minor 5, warning 10, block 50
        let thresholds = TooManyLinesThresholds::default();
        let cases = [
            (0, None),
            (4, None),
            (5, Some(Severity::Slight)),
            (9, Some(Severity::Slight)),
            (10, Some(Severity::Warning)),
            (50, Some(Severity::Warning)),
            (51, Some(Severity::Block)),
        ];

        for (ratio_pct, expected) in cases {
            assert_eq!(
                ratio_severity(ratio_pct, &thresholds),
                expected,
                "ratio {ratio_pct} %"
            );
        }
    }

    #[test]
    fn minutes_severity_for_capacity_one() {
        // default simultaneous_leeway is 5 minutes
        let thresholds = TooManyLinesThresholds::default();
        let cases = [
            (0, None),
            (5, Some(Severity::Slight)),
            (10, Some(Severity::Block)),
        ];

        for (overloaded, expected) in cases {
            assert_eq!(
                minutes_severity(Minutes::new(overloaded), &thresholds),
                expected,
                "{overloaded} overloaded minutes"
            );
        }
    }

    /// A project with one bar, returning the bar's id.
    fn project_with_bar(
        capacity: u32,
        overload: Option<TooManyLinesThresholds>,
    ) -> (Project, LocationId) {
        let date = NaiveDate::from_ymd_opt(2026, 5, 23).unwrap();
        let mut project = Project::new(String::from("Jyväsmetro"), date);
        let bar = Location::new(
            String::from("Hiisi"),
            LocationKind::Bar {
                capacity: NonZeroU32::new(capacity).unwrap(),
                area_id: AreaId::new(),
                overload,
            },
            String::from("Lutakonaukio 3, 40100 Jyväskylä"),
            None,
            None,
        );
        let id = bar.id;
        project.locations.push(bar);
        (project, id)
    }

    fn visit_at(bar: LocationId, index: usize, arrival: &str, departure: &str) -> ComputedVisit {
        ComputedVisit {
            location_id: bar,
            index,
            ..visit(arrival, departure)
        }
    }

    #[test]
    fn capacity_one_overlap_blocks_both_lines() {
        let (project, bar) = project_with_bar(1, None);
        // 10 minutes overlap 18:30–18:40
        let first = visit_at(bar, 0, "18:00", "18:40");
        let second = visit_at(bar, 2, "18:30", "19:10");
        let timelines = BTreeMap::from([(bar, vec![first.clone(), second.clone()])]);

        let expected: Vec<Violation> = [first, second]
            .iter()
            .map(|visit| Violation {
                severity: Severity::Block,
                kind: ViolationKind::TooManyLines {
                    line_id: visit.line_id,
                    location_id: bar,
                    visit_index: visit.index,
                    overloaded: Minutes::new(10),
                    ratio_pct: 25,
                },
            })
            .collect();
        assert_eq!(check_capacity(&project, &timelines), expected);
    }

    fn severities(project: &Project, bar: LocationId) -> Vec<Severity> {
        // capacity 2, three lines present 18:30–18:40: 10 of 40 minutes = 25 %
        let visits = vec![
            visit_at(bar, 0, "18:00", "18:40"),
            visit_at(bar, 0, "18:30", "19:10"),
            visit_at(bar, 0, "18:30", "19:10"),
        ];
        let timelines = BTreeMap::from([(bar, visits)]);
        check_capacity(project, &timelines)
            .iter()
            .map(|violation| violation.severity)
            .collect()
    }

    #[test]
    fn bar_uses_global_thresholds_by_default() {
        let (project, bar) = project_with_bar(2, None);

        assert_eq!(severities(&project, bar), vec![Severity::Warning; 3]);
    }

    #[test]
    fn bar_uses_its_own_thresholds_when_set() {
        let own = TooManyLinesThresholds {
            block: 20,
            ..TooManyLinesThresholds::default()
        };
        let (project, bar) = project_with_bar(2, Some(own));

        assert_eq!(severities(&project, bar), vec![Severity::Block; 3]);
    }
}
