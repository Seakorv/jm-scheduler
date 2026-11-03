use std::collections::BTreeMap;

use crate::{ComputedLine, ComputedVisit, LocationId};

/// Goes through computed lines and adds their visits to bars to form bar schedules. The returned `BTreeMap` is sorted by arrival time.
pub fn bar_timelines(c_lines: &[ComputedLine]) -> BTreeMap<LocationId, Vec<ComputedVisit>> {
    let mut bar_info: BTreeMap<LocationId, Vec<ComputedVisit>> = BTreeMap::new();

    for c_line in c_lines {
        for visit in &c_line.visits {
            bar_info
                .entry(visit.location_id)
                .or_default()
                .push(visit.clone());
        }
    }

    for bar in bar_info.values_mut() {
        bar.sort_by_key(|visit| visit.arrival);
    }

    bar_info
}

// tests by Claude
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineId, LineStatus, Minutes, TimeLeft, TimeOfDay};

    fn time(s: &str) -> TimeOfDay {
        s.parse().unwrap()
    }

    fn computed_visit(line_id: LineId, location_id: LocationId, arrival: &str) -> ComputedVisit {
        let arrival = time(arrival);
        ComputedVisit {
            line_id,
            location_id,
            index: 0,
            arrival,
            departure: arrival.checked_add(Minutes::new(40)).unwrap(),
            travel_in: Minutes::new(10),
        }
    }

    // bar_timelines only reads the visits, so the status doesn't matter
    fn computed_line(line_id: LineId, visits: Vec<ComputedVisit>) -> ComputedLine {
        ComputedLine {
            line_id,
            visits,
            status: LineStatus::Complete {
                end_arrival: time("21:00"),
                travel_to_end: Minutes::new(10),
                total: Minutes::new(300),
                time_left: TimeLeft::Zero,
            },
        }
    }

    #[test]
    fn bar_timelines_groups_by_bar_sorted_by_arrival() {
        let teerenpeli = LocationId::new();
        let hiisi = LocationId::new();
        let early = LineId::new();
        let late = LineId::new();

        // the later line comes first, so the result must be sorted
        let lines = [
            computed_line(
                late,
                vec![
                    computed_visit(late, teerenpeli, "17:00"),
                    computed_visit(late, hiisi, "18:00"),
                ],
            ),
            computed_line(early, vec![computed_visit(early, teerenpeli, "16:10")]),
        ];

        let timelines = bar_timelines(&lines);

        assert_eq!(timelines.len(), 2);
        let teerenpeli_lines: Vec<LineId> = timelines[&teerenpeli]
            .iter()
            .map(|visit| visit.line_id)
            .collect();
        assert_eq!(teerenpeli_lines, vec![early, late]);
        assert_eq!(timelines[&hiisi].len(), 1);
    }
}
