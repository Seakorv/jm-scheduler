use crate::{ComputedVisit, TimeOfDay};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Change {
    Leave,
    Arrive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BarEvent {
    time: TimeOfDay,
    change: Change,
}

/// A stretch of time during which the number of lines at a bar stays the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Segment {
    pub start: TimeOfDay,
    pub end: TimeOfDay,
    pub count: u32,
}

/// Makes bar visits into two events about departures and arrivals. Then puts them into a list and sorts them.
/// Sorting by time, departures first.
fn make_bar_events(visits: &[ComputedVisit]) -> Vec<BarEvent> {
    let mut events = Vec::new();

    for visit in visits {
        let arrival_event = BarEvent {
            time: visit.arrival,
            change: Change::Arrive,
        };
        let departure_event = BarEvent {
            time: visit.departure,
            change: Change::Leave,
        };
        // Skipping zero minute visits
        if arrival_event.time.minutes() != departure_event.time.minutes() {
            events.extend([arrival_event, departure_event]);
        }
    }

    events.sort();
    events
}

/// Goes through bar events and makes segments from those
fn sweep_events(bar_events: &[BarEvent]) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut count: u32 = 0;

    for pair in bar_events.windows(2) {
        let (event, next) = (&pair[0], &pair[1]);
        match event.change {
            Change::Arrive => count += 1,
            Change::Leave => count -= 1,
        }
        if event.time < next.time {
            segments.push(Segment {
                start: event.time,
                end: next.time,
                count,
            });
        }
    }

    segments
}

/// Splits one bar's visits into segments of how many lines are present, in time order.
pub fn bar_segments(visits: &[ComputedVisit]) -> Vec<Segment> {
    sweep_events(&make_bar_events(visits))
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineId, LocationId, Minutes};

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

    fn event(at: &str, change: Change) -> BarEvent {
        BarEvent {
            time: time(at),
            change,
        }
    }

    #[test]
    fn back_to_back_visits_leave_before_arrive() {
        // the later visit comes first, so the events must be sorted
        let visits = [visit("18:40", "19:20"), visit("18:00", "18:40")];

        assert_eq!(
            make_bar_events(&visits),
            vec![
                event("18:00", Change::Arrive),
                event("18:40", Change::Leave),
                event("18:40", Change::Arrive),
                event("19:20", Change::Leave),
            ]
        );
    }

    #[test]
    fn zero_minute_visit_makes_no_events() {
        assert_eq!(make_bar_events(&[visit("18:00", "18:00")]), vec![]);
    }

    fn segment(start: &str, end: &str, count: u32) -> Segment {
        Segment {
            start: time(start),
            end: time(end),
            count,
        }
    }

    fn segments_of(visits: &[ComputedVisit]) -> Vec<Segment> {
        sweep_events(&make_bar_events(visits))
    }

    #[test]
    fn three_overlapping_visits() {
        let visits = [
            visit("18:00", "18:40"),
            visit("18:10", "18:50"),
            visit("18:20", "19:00"),
        ];

        assert_eq!(
            segments_of(&visits),
            vec![
                segment("18:00", "18:10", 1),
                segment("18:10", "18:20", 2),
                segment("18:20", "18:40", 3),
                segment("18:40", "18:50", 2),
                segment("18:50", "19:00", 1),
            ]
        );
    }

    #[test]
    fn same_time_arrivals_make_one_segment() {
        let visits = [
            visit("18:00", "18:40"),
            visit("18:00", "18:40"),
            visit("18:00", "18:40"),
        ];

        assert_eq!(segments_of(&visits), vec![segment("18:00", "18:40", 3)]);
    }

    #[test]
    fn idle_time_is_a_zero_segment() {
        let visits = [visit("18:00", "18:30"), visit("18:50", "19:20")];

        assert_eq!(
            segments_of(&visits),
            vec![
                segment("18:00", "18:30", 1),
                segment("18:30", "18:50", 0),
                segment("18:50", "19:20", 1),
            ]
        );
    }
}
