use crate::{Line, LineId, LocationId, Minutes, Settings, TimeOfDay, TravelTable, Visit};
use serde::Serialize;

/// The one visit event which is computed and not stored.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedVisit {
    pub line_id: LineId,
    pub location_id: LocationId,
    pub index: usize,
    pub arrival: TimeOfDay,
    pub departure: TimeOfDay,
    pub travel_in: Minutes,
}

/// One line's schedule computed
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedLine {
    pub line_id: LineId,
    pub visits: Vec<ComputedVisit>,
    pub status: LineStatus,
}

/// How much time is left to be scheduled. `Short` means there is time left, `Over` means it is over the
/// maximum line time.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TimeLeft {
    Short(Minutes),
    Over(Minutes),
    Zero,
}

/// Status for `Line` indicating if it is complete or not
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LineStatus {
    Complete {
        end_arrival: TimeOfDay,
        travel_to_end: Minutes,
        total: Minutes,
        time_left: TimeLeft,
    },
    Incomplete {
        from_index: usize,
        reason: IncompleteReason,
    },
}

/// The reason why a `Line` is not complete.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum IncompleteReason {
    MissingTravelTime { from: LocationId, to: LocationId },
    PastMidnight,
}

/// Calculates a single computed visit
fn compute_visit(
    visit: &Visit,
    index: usize,
    line_id: LineId,
    from: LocationId,
    prev_departure: TimeOfDay,
    travel: &TravelTable,
    step: Minutes,
) -> Result<ComputedVisit, IncompleteReason> {
    // Calculating the travel time to the visit, rounded.
    let travel_in = travel.rounded(from, visit.location_id, step).ok_or(
        IncompleteReason::MissingTravelTime {
            from,
            to: visit.location_id,
        },
    )?;
    // calculating the arrival's `TimeOfDay`
    let arrival = prev_departure
        .checked_add(travel_in)
        .ok_or(IncompleteReason::PastMidnight)?;
    // calculating departure
    let departure = arrival
        .checked_add(visit.stay_time)
        .ok_or(IncompleteReason::PastMidnight)?;
    // returning the computed visit
    Ok(ComputedVisit {
        line_id,
        location_id: visit.location_id,
        index,
        arrival,
        departure,
        travel_in,
    })
}

/// Calculates the lines visits and travel times into `ComputedLine`
pub fn compute_line(
    line: &Line,
    travel_table: &TravelTable,
    settings: &Settings,
    start: LocationId,
    _end: LocationId,
) -> ComputedLine {
    let mut visits: Vec<ComputedVisit> = Vec::new();
    let mut from = start;
    let mut prev_departure = line.departure;
    for (i, visit) in line.visits.iter().enumerate() {
        let cvisit = compute_visit(
            visit,
            i,
            line.id,
            from,
            prev_departure,
            travel_table,
            settings.rounding_step,
        );

        match cvisit {
            Ok(value) => {
                from = value.location_id;
                prev_departure = value.departure;
                visits.push(value);
            }
            Err(incomplete) => {
                return ComputedLine {
                    line_id: line.id,
                    visits,
                    status: LineStatus::Incomplete {
                        from_index: i,
                        reason: incomplete,
                    },
                };
            }
        }
    }

    ComputedLine {
        line_id: line.id,
        visits,
        status: LineStatus::Incomplete {
            from_index: 0,
            reason: IncompleteReason::PastMidnight,
        },
    }
}

// tests by Claude
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Line, Settings, TravelTable, TravelTime, Visit};

    fn time(s: &str) -> TimeOfDay {
        s.parse().unwrap()
    }

    fn travel_time(from_id: LocationId, to_id: LocationId, minutes: u32) -> TravelTime {
        TravelTime {
            from_id,
            to_id,
            minutes: Minutes::new(minutes),
            distance_km: None,
        }
    }

    #[test]
    fn first_visit_times() {
        let start = LocationId::new();
        let bar = LocationId::new();
        let end = LocationId::new();
        // 8 raw minutes rounds up to 10
        let table = TravelTable::new(&[travel_time(start, bar, 8), travel_time(bar, end, 10)]);

        let mut line = Line::new(1, None, false, time("16:00"), None);
        line.visits.push(Visit {
            location_id: bar,
            stay_time: Minutes::new(40),
        });

        let computed = compute_line(&line, &table, &Settings::default(), start, end);

        assert_eq!(
            computed.visits[0],
            ComputedVisit {
                line_id: line.id,
                location_id: bar,
                index: 0,
                arrival: time("16:10"),
                departure: time("16:50"),
                travel_in: Minutes::new(10),
            }
        );
    }

    #[test]
    fn all_visits_times() {
        let start = LocationId::new();
        let bars = [LocationId::new(), LocationId::new(), LocationId::new()];
        let end = LocationId::new();
        let table = TravelTable::new(&[
            travel_time(start, bars[0], 10),
            // 13 raw minutes rounds up to 15
            travel_time(bars[0], bars[1], 13),
            travel_time(bars[1], bars[2], 5),
            travel_time(bars[2], end, 10),
        ]);

        let mut line = Line::new(1, None, false, time("16:00"), None);
        for (bar, stay) in bars.iter().zip([40, 30, 45]) {
            line.visits.push(Visit {
                location_id: *bar,
                stay_time: Minutes::new(stay),
            });
        }

        let computed = compute_line(&line, &table, &Settings::default(), start, end);

        let times: Vec<(TimeOfDay, TimeOfDay, Minutes)> = computed
            .visits
            .iter()
            .map(|visit| (visit.arrival, visit.departure, visit.travel_in))
            .collect();
        assert_eq!(
            times,
            vec![
                (time("16:10"), time("16:50"), Minutes::new(10)),
                (time("17:05"), time("17:35"), Minutes::new(15)),
                (time("17:40"), time("18:25"), Minutes::new(5)),
            ]
        );
        assert_eq!(computed.visits[2].index, 2);
        assert_eq!(computed.visits[2].location_id, bars[2]);
    }
}
