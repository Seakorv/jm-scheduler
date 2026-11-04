use std::collections::HashSet;

use crate::{
    AreaId, ComputedLine, IncompleteReason, Line, LineStatus, LocationId, LocationKind, Project,
    Settings,
    Severity::{self, Warning},
    TravelTable, Violation, ViolationKind,
};

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

/// Checks if some travel times are missing or line is past midnight
pub fn check_incomplete_line(line: &ComputedLine) -> Option<Violation> {
    match line.status {
        LineStatus::Complete { .. } => None,
        LineStatus::Incomplete {
            reason: IncompleteReason::MissingTravelTime { from, to },
            ..
        } => Some(Violation {
            severity: Severity::Warning,
            kind: ViolationKind::MissingTravelTime {
                line_id: line.id,
                from,
                to,
            },
        }),
        LineStatus::Incomplete {
            from_index,
            reason: IncompleteReason::PastMidnight,
        } => Some(Violation {
            severity: Warning,
            kind: ViolationKind::PastMidnight {
                line_id: line.id,
                visit_index: from_index,
            },
        }),
    }
}

/// Gives the area id of a bar
fn area_of_bar(project: &Project, id: LocationId) -> Option<AreaId> {
    let location = project.location(id)?;
    match location.kind {
        LocationKind::Bar { area_id, .. } => Some(area_id),
        _ => None,
    }
}

/// checks lines visits if any of them has two areas in a row. If yes, it checks if any of them
/// have too short of a travel time, configured in project settings.
pub fn check_same_area(
    line: &Line,
    project: &Project,
    travel_table: &TravelTable,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (i, pair) in line.visits.windows(2).enumerate() {
        let (a, b) = (&pair[0], &pair[1]);

        let (Some(area_a), Some(area_b)) = (
            area_of_bar(project, a.location_id),
            area_of_bar(project, b.location_id),
        ) else {
            continue;
        };
        if area_a != area_b {
            continue;
        }

        let Some(minutes) =
            travel_table.rounded(a.location_id, b.location_id, project.settings.rounding_step)
        else {
            continue;
        };

        if minutes < project.settings.same_area_min_travel {
            violations.push(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::SameAreaInARow {
                    line_id: line.id,
                    visit_index: i + 1,
                    area_id: area_a,
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
    use crate::{
        Area, LineId, Location, LocationId, Minutes, TimeLeft, TimeOfDay, TravelTime, Visit,
    };
    use chrono::NaiveDate;
    use std::num::NonZeroU32;

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

    fn computed_line(status: LineStatus) -> ComputedLine {
        ComputedLine {
            id: LineId::new(),
            visits: vec![],
            status,
        }
    }

    #[test]
    fn complete_line_has_nothing_missing() {
        let line = computed_line(LineStatus::Complete {
            end_arrival: TimeOfDay::new(1260).unwrap(),
            travel_to_end: Minutes::new(10),
            total: Minutes::new(300),
            time_left: TimeLeft::Zero,
        });

        assert_eq!(check_incomplete_line(&line), None);
    }

    #[test]
    fn missing_travel_time_warns_with_both_locations() {
        let from = LocationId::new();
        let to = LocationId::new();
        let line = computed_line(LineStatus::Incomplete {
            from_index: 2,
            reason: IncompleteReason::MissingTravelTime { from, to },
        });

        assert_eq!(
            check_incomplete_line(&line),
            Some(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::MissingTravelTime {
                    line_id: line.id,
                    from,
                    to,
                },
            })
        );
    }

    #[test]
    fn past_midnight_warns_with_visit_index() {
        let line = computed_line(LineStatus::Incomplete {
            from_index: 3,
            reason: IncompleteReason::PastMidnight,
        });

        assert_eq!(
            check_incomplete_line(&line),
            Some(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::PastMidnight {
                    line_id: line.id,
                    visit_index: 3,
                },
            })
        );
    }

    fn bar_in(project: &mut Project, area_id: AreaId) -> LocationId {
        let bar = Location::new(
            String::from("Bar"),
            LocationKind::Bar {
                capacity: NonZeroU32::new(1).unwrap(),
                area_id,
                overload: None,
            },
            String::from("Kauppakatu 32, 40100 Jyväskylä"),
            None,
            None,
        );
        let id = bar.id;
        project.locations.push(bar);
        id
    }

    /// A line visiting two bars in a row, with the given travel time between them.
    fn two_bars(same_area: bool, travel_minutes: u32) -> (Project, TravelTable, Line) {
        let date = NaiveDate::from_ymd_opt(2026, 5, 23).unwrap();
        let mut project = Project::new(String::from("Jyväsmetro"), date);
        let keskusta = Area::new(String::from("Keskusta"));
        let lutakko = Area::new(String::from("Lutakko"));

        let first = bar_in(&mut project, keskusta.id);
        let second_area = if same_area { keskusta.id } else { lutakko.id };
        let second = bar_in(&mut project, second_area);
        project.areas.push(keskusta);
        project.areas.push(lutakko);

        let table = TravelTable::new(&[TravelTime {
            from_id: first,
            to_id: second,
            minutes: Minutes::new(travel_minutes),
            distance_km: None,
        }]);
        let line = line_visiting(&[first, second]);

        (project, table, line)
    }

    // default same_area_min_travel is 10
    #[test]
    fn same_area_with_short_travel_warns() {
        let (project, table, line) = two_bars(true, 5);

        assert_eq!(
            check_same_area(&line, &project, &table),
            vec![Violation {
                severity: Severity::Warning,
                kind: ViolationKind::SameAreaInARow {
                    line_id: line.id,
                    visit_index: 1,
                    area_id: project.areas[0].id,
                },
            }]
        );
    }

    #[test]
    fn same_area_with_enough_travel_is_ok() {
        let (project, table, line) = two_bars(true, 10);

        assert_eq!(check_same_area(&line, &project, &table), vec![]);
    }

    #[test]
    fn different_areas_are_ok() {
        let (project, table, line) = two_bars(false, 5);

        assert_eq!(check_same_area(&line, &project, &table), vec![]);
    }
}
