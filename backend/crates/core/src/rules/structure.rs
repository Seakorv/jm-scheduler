use std::collections::HashSet;

use crate::{Line, Settings, Severity, Violation, ViolationKind};

/// Checks the stop count of a line and gives warnings if they are not correct
pub fn check_stop_count(line: &Line, settings: &Settings) -> Option<Violation> {
    let count = u32::try_from(line.visits.len()).unwrap_or(u32::MAX);

    match count {
        0 => Some(Violation {
            severity: Severity::Warning,
            kind: ViolationKind::NoVisits { line_id: line.id },
        }),
        n if !line.is_vip && n != settings.stops_per_line => Some(Violation {
            severity: Severity::Warning,
            kind: ViolationKind::WrongStopCount {
                line_id: line.id,
                count,
                expected: settings.stops_per_line,
            },
        }),
        _ => None,
    }
}

/// Checks if there are duplicate bars in line visits.
pub fn check_duplicate_bar(line: &Line) -> Option<Violation> {
    let mut duplicate = HashSet::new();
    for visit in &line.visits {
        if !duplicate.insert(visit.location_id) {
            return Some(Violation {
                severity: Severity::Block,
                kind: ViolationKind::DuplicateBar {
                    line_id: line.id,
                    location_id: visit.location_id,
                },
            });
        }
    }
    None
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LocationId, Minutes, TimeOfDay, Visit};

    fn line_with_stops(stops: usize, is_vip: bool) -> Line {
        let mut line = Line::new(1, None, is_vip, TimeOfDay::new(960).unwrap(), None);
        for _ in 0..stops {
            line.visits.push(Visit {
                location_id: LocationId::new(),
                stay_time: Minutes::new(40),
            });
        }
        line
    }

    // default stops_per_line is 5
    #[test]
    fn regular_line_with_right_count_is_ok() {
        assert_eq!(
            check_stop_count(&line_with_stops(5, false), &Settings::default()),
            None
        );
    }

    #[test]
    fn regular_line_with_wrong_count_warns() {
        let line = line_with_stops(4, false);

        assert_eq!(
            check_stop_count(&line, &Settings::default()),
            Some(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::WrongStopCount {
                    line_id: line.id,
                    count: 4,
                    expected: 5,
                },
            })
        );
    }

    #[test]
    fn vip_line_with_any_count_is_ok() {
        let settings = Settings::default();

        assert_eq!(check_stop_count(&line_with_stops(5, true), &settings), None);
        assert_eq!(check_stop_count(&line_with_stops(3, true), &settings), None);
    }

    #[test]
    fn empty_line_warns_no_visits_vip_or_not() {
        for is_vip in [false, true] {
            let line = line_with_stops(0, is_vip);

            assert_eq!(
                check_stop_count(&line, &Settings::default()),
                Some(Violation {
                    severity: Severity::Warning,
                    kind: ViolationKind::NoVisits { line_id: line.id },
                })
            );
        }
    }

    fn line_visiting(bars: &[LocationId]) -> Line {
        let mut line = Line::new(1, None, false, TimeOfDay::new(960).unwrap(), None);
        for &location_id in bars {
            line.visits.push(Visit {
                location_id,
                stay_time: Minutes::new(40),
            });
        }
        line
    }

    #[test]
    fn different_bars_are_ok() {
        let line = line_visiting(&[LocationId::new(), LocationId::new(), LocationId::new()]);

        assert_eq!(check_duplicate_bar(&line), None);
    }

    #[test]
    fn same_bar_twice_blocks() {
        let hiisi = LocationId::new();
        let line = line_visiting(&[hiisi, LocationId::new(), hiisi]);

        assert_eq!(
            check_duplicate_bar(&line),
            Some(Violation {
                severity: Severity::Block,
                kind: ViolationKind::DuplicateBar {
                    line_id: line.id,
                    location_id: hiisi,
                },
            })
        );
    }
}
