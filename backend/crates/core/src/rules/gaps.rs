use std::collections::BTreeMap;

use crate::{
    ComputedVisit, LocationId, Minutes, Segment, Settings, Severity, Violation, ViolationKind,
    bar_segments,
};

/// Checks the gaps in bar visits from segments
pub fn bar_gaps(segments: &[Segment], bar_gap_time: Minutes) -> Vec<Segment> {
    segments
        .iter()
        .filter(|s| s.count == 0 && s.duration() > bar_gap_time)
        .copied()
        .collect()
}

/// Checks if there are gaps and gives violations accordingy. 1 gap is a slight warning, 2 gaps is a warning.
pub fn bar_gap_violations(
    timelines: &BTreeMap<LocationId, Vec<ComputedVisit>>,
    settings: &Settings,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (location_id, visits) in timelines {
        let segments = bar_segments(visits);
        let gap_segments = bar_gaps(&segments, settings.bar_gap_time);

        let gaps_len: u32 = u32::try_from(gap_segments.len()).unwrap_or(u32::MAX);

        let Some(longest) = gap_segments.iter().map(|gap| gap.duration()).max() else {
            continue;
        };

        if gaps_len == 1 {
            violations.push(Violation {
                severity: Severity::Slight,
                kind: ViolationKind::BarGaps {
                    location_id: *location_id,
                    count: gaps_len,
                    longest,
                },
            });
        }
        if gaps_len >= 2 {
            violations.push(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::BarGaps {
                    location_id: *location_id,
                    count: gaps_len,
                    longest,
                },
            });
        }
    }
    violations
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineId, TimeOfDay};

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

    fn gaps_of(visits: &[ComputedVisit]) -> Vec<Segment> {
        bar_gaps(&bar_segments(visits), Minutes::new(10))
    }

    #[test]
    fn overlapping_visits_have_no_gaps() {
        let visits = [visit("18:00", "18:40"), visit("18:30", "19:10")];

        assert_eq!(gaps_of(&visits), []);
    }

    #[test]
    fn one_gap_between_two_visits() {
        let visits = [visit("18:00", "18:40"), visit("19:00", "19:40")];

        let gaps = gaps_of(&visits);

        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].start, time("18:40"));
        assert_eq!(gaps[0].end, time("19:00"));
    }

    #[test]
    fn two_gaps_between_three_visits() {
        let visits = [
            visit("18:00", "18:40"),
            visit("19:00", "19:40"),
            visit("20:00", "20:40"),
        ];

        assert_eq!(gaps_of(&visits).len(), 2);
    }

    #[test]
    fn gap_of_exactly_bar_gap_time_is_not_a_gap() {
        let visits = [visit("18:00", "18:40"), visit("18:50", "19:30")];

        assert_eq!(gaps_of(&visits), []);
    }

    #[test]
    fn short_visit_inside_a_long_one_has_no_gaps() {
        let visits = [visit("18:00", "19:30"), visit("18:30", "18:40")];

        assert_eq!(gaps_of(&visits), []);
    }

    fn violations_of(visits: Vec<ComputedVisit>) -> (LocationId, Vec<Violation>) {
        let bar = LocationId::new();
        let timelines = BTreeMap::from([(bar, visits)]);

        (bar, bar_gap_violations(&timelines, &Settings::default()))
    }

    #[test]
    fn bar_without_gaps_has_no_violation() {
        let (_, violations) = violations_of(vec![visit("18:00", "18:40"), visit("18:30", "19:10")]);

        assert_eq!(violations, []);
    }

    #[test]
    fn one_gap_is_slight() {
        let (bar, violations) =
            violations_of(vec![visit("18:00", "18:40"), visit("19:00", "19:40")]);

        assert_eq!(
            violations,
            [Violation {
                severity: Severity::Slight,
                kind: ViolationKind::BarGaps {
                    location_id: bar,
                    count: 1,
                    longest: Minutes::new(20),
                },
            }]
        );
    }

    #[test]
    fn two_gaps_are_a_warning_with_the_longest_gap() {
        let (bar, violations) = violations_of(vec![
            visit("18:00", "18:40"),
            visit("19:00", "19:40"),
            visit("20:10", "20:50"),
        ]);

        assert_eq!(
            violations,
            [Violation {
                severity: Severity::Warning,
                kind: ViolationKind::BarGaps {
                    location_id: bar,
                    count: 2,
                    longest: Minutes::new(30),
                },
            }]
        );
    }

    #[test]
    fn only_bars_with_gaps_get_a_violation() {
        let gappy = LocationId::new();
        let busy = LocationId::new();
        let timelines = BTreeMap::from([
            (
                gappy,
                vec![visit("18:00", "18:40"), visit("19:00", "19:40")],
            ),
            (busy, vec![visit("18:00", "18:40"), visit("18:30", "19:10")]),
        ]);

        let violations = bar_gap_violations(&timelines, &Settings::default());

        assert_eq!(violations.len(), 1);
        assert!(matches!(
            violations[0].kind,
            ViolationKind::BarGaps { location_id, .. } if location_id == gappy
        ));
    }
}
