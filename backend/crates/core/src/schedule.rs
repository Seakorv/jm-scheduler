use std::cmp::Ordering;

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

/// Calculating the final travel time to end and the time of arrival to end
fn compute_end(
    from: LocationId,
    prev_departure: TimeOfDay,
    end: LocationId,
    travel_table: &TravelTable,
    step: Minutes,
) -> Result<(Minutes, TimeOfDay), IncompleteReason> {
    let travel_to_end = travel_table
        .rounded(from, end, step)
        .ok_or(IncompleteReason::MissingTravelTime { from, to: end })?;

    let end_arrival = prev_departure
        .checked_add(travel_to_end)
        .ok_or(IncompleteReason::PastMidnight)?;

    Ok((travel_to_end, end_arrival))
}

/// Calcualtes how much time is still left to allocate to a `Line`
fn time_left(total: Minutes, line_duration: Minutes) -> TimeLeft {
    let time_left = Minutes::new(total.amount().abs_diff(line_duration.amount()));
    match total.cmp(&line_duration) {
        Ordering::Equal => TimeLeft::Zero,
        Ordering::Less => TimeLeft::Short(time_left),
        Ordering::Greater => TimeLeft::Over(time_left),
    }
}

/// Calcualtes the total minutes of a `Line` from departure time to end arrival time
fn total_time(end_arrival: TimeOfDay, line_departure: TimeOfDay) -> Minutes {
    Minutes::new(end_arrival.minutes() - line_departure.minutes())
}

/// Calculates the lines visits and travel times into `ComputedLine`
pub fn compute_line(
    line: &Line,
    travel_table: &TravelTable,
    settings: &Settings,
    start: LocationId,
    end: LocationId,
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

    let visits_len = visits.len();

    match compute_end(
        from,
        prev_departure,
        end,
        travel_table,
        settings.rounding_step,
    ) {
        Ok((travel_to_end, end_arrival)) => {
            let total = total_time(end_arrival, line.departure);
            ComputedLine {
                line_id: line.id,
                visits,
                status: LineStatus::Complete {
                    end_arrival,
                    travel_to_end,
                    total,
                    time_left: time_left(total, settings.line_duration),
                },
            }
        }
        Err(reason) => ComputedLine {
            line_id: line.id,
            visits,
            status: LineStatus::Incomplete {
                from_index: visits_len,
                reason,
            },
        },
    }
}

// tests by Claude
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Line, Settings, TravelTable, TravelTime, Visit};
    use proptest::prelude::*;

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

    #[test]
    fn time_left_zero_short_and_over() {
        let duration = Minutes::new(300);

        assert_eq!(time_left(Minutes::new(300), duration), TimeLeft::Zero);
        assert_eq!(
            time_left(Minutes::new(290), duration),
            TimeLeft::Short(Minutes::new(10))
        );
        assert_eq!(
            time_left(Minutes::new(305), duration),
            TimeLeft::Over(Minutes::new(5))
        );
    }

    #[test]
    fn line_without_visits_goes_straight_to_end() {
        let start = LocationId::new();
        let end = LocationId::new();
        let table = TravelTable::new(&[travel_time(start, end, 10)]);
        let line = Line::new(1, None, false, time("16:00"), None);

        let computed = compute_line(&line, &table, &Settings::default(), start, end);

        assert_eq!(computed.visits, vec![]);
        assert_eq!(
            computed.status,
            LineStatus::Complete {
                end_arrival: time("16:10"),
                travel_to_end: Minutes::new(10),
                total: Minutes::new(10),
                time_left: TimeLeft::Short(Minutes::new(290)),
            }
        );
    }

    #[test]
    fn missing_travel_between_visits_stops_the_line() {
        let start = LocationId::new();
        let bars = [LocationId::new(), LocationId::new(), LocationId::new()];
        let end = LocationId::new();
        // bars[0] -> bars[1] is missing
        let table = TravelTable::new(&[
            travel_time(start, bars[0], 10),
            travel_time(bars[1], bars[2], 10),
            travel_time(bars[2], end, 10),
        ]);

        let mut line = Line::new(1, None, false, time("16:00"), None);
        for bar in bars {
            line.visits.push(Visit {
                location_id: bar,
                stay_time: Minutes::new(40),
            });
        }

        let computed = compute_line(&line, &table, &Settings::default(), start, end);

        assert_eq!(computed.visits.len(), 1);
        assert_eq!(
            computed.status,
            LineStatus::Incomplete {
                from_index: 1,
                reason: IncompleteReason::MissingTravelTime {
                    from: bars[0],
                    to: bars[1],
                },
            }
        );
    }

    #[test]
    fn missing_travel_to_end_keeps_all_visits() {
        let start = LocationId::new();
        let bar = LocationId::new();
        let end = LocationId::new();
        let table = TravelTable::new(&[travel_time(start, bar, 10)]);

        let mut line = Line::new(1, None, false, time("16:00"), None);
        line.visits.push(Visit {
            location_id: bar,
            stay_time: Minutes::new(40),
        });

        let computed = compute_line(&line, &table, &Settings::default(), start, end);

        assert_eq!(computed.visits.len(), 1);
        assert_eq!(
            computed.status,
            LineStatus::Incomplete {
                from_index: 1,
                reason: IncompleteReason::MissingTravelTime { from: bar, to: end },
            }
        );
    }

    #[test]
    fn visit_past_midnight_stops_the_line() {
        let start = LocationId::new();
        let bar = LocationId::new();
        let end = LocationId::new();
        let table = TravelTable::new(&[travel_time(start, bar, 10), travel_time(bar, end, 10)]);

        // arrives 23:40, would leave 00:20
        let mut line = Line::new(1, None, false, time("23:30"), None);
        line.visits.push(Visit {
            location_id: bar,
            stay_time: Minutes::new(40),
        });

        let computed = compute_line(&line, &table, &Settings::default(), start, end);

        assert_eq!(computed.visits, vec![]);
        assert_eq!(
            computed.status,
            LineStatus::Incomplete {
                from_index: 0,
                reason: IncompleteReason::PastMidnight,
            }
        );
    }

    // Random lines that always come out complete: every leg has a travel time, and departures
    // 08:00–14:00 + at most 7 × (30 min travel + 50 min stay) + 30 min to the end never pass midnight.
    proptest! {
        #[test]
        fn computed_line_adds_up(
            departure in 480u32..=840,
            stops in prop::collection::vec((0u32..=30, 5u32..=50), 1..=7),
            end_travel in 0u32..=30,
        ) {
            let start = LocationId::new();
            let end = LocationId::new();
            let mut line = Line::new(1, None, false, TimeOfDay::new(departure).unwrap(), None);
            let mut travel_times = Vec::new();
            let mut from = start;
            for &(travel, stay) in &stops {
                let bar = LocationId::new();
                travel_times.push(travel_time(from, bar, travel));
                line.visits.push(Visit {
                    location_id: bar,
                    stay_time: Minutes::new(stay),
                });
                from = bar;
            }
            travel_times.push(travel_time(from, end, end_travel));
            let table = TravelTable::new(&travel_times);

            let computed = compute_line(&line, &table, &Settings::default(), start, end);

            let LineStatus::Complete { travel_to_end, total, .. } = computed.status else {
                panic!("line should be complete");
            };
            prop_assert_eq!(computed.visits.len(), stops.len());

            // arrivals strictly increase
            for pair in computed.visits.windows(2) {
                prop_assert!(pair[1].arrival > pair[0].arrival);
            }

            // total == all travel + all stays
            let travel_sum: u32 = computed.visits.iter().map(|visit| visit.travel_in.amount()).sum();
            let stay_sum: u32 = stops.iter().map(|&(_, stay)| stay).sum();
            prop_assert_eq!(total.amount(), travel_sum + travel_to_end.amount() + stay_sum);
        }
    }
}
