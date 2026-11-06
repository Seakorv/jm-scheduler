use serde::Serialize;

use crate::{ComputedVisit, Minutes, Segment, TimeOfDay, bar_gaps, bar_segments};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarStats {
    pub lines_count: u32,
    pub first_arrival: TimeOfDay,
    pub last_departure: TimeOfDay,
    pub gaps: Vec<Segment>,
    pub pacing: Option<f64>,
    pub longest_gap: Option<Minutes>,
}

impl BarStats {
    pub fn from_visits(visits: &[ComputedVisit], bar_gap_time: Minutes) -> Option<Self> {
        let lines_count = u32::try_from(visits.len()).unwrap_or(u32::MAX);
        let segments = bar_segments(visits);
        let gaps = bar_gaps(&segments, bar_gap_time);
        let longest_gap = gaps.iter().map(|gap| gap.duration()).max();
        let first_arrival = visits.iter().map(|v| v.arrival).min()?;
        let last_departure = visits.iter().map(|v| v.departure).max()?;

        // arrivals should be already sorted when taking from visits
        let arrivals: Vec<TimeOfDay> = visits.iter().map(|v| v.arrival).collect();
        let pacing = pacing(&arrivals);

        Some(Self {
            lines_count,
            first_arrival,
            last_departure,
            gaps,
            pacing,
            longest_gap,
        })
    }
}

/// Calculates the pacing as a standard deviation. Arrivals is expected to be sorted
fn pacing(arrivals: &[TimeOfDay]) -> Option<f64> {
    if arrivals.len() < 2 {
        return None;
    }

    let intervals: Vec<u32> = arrivals
        .windows(2)
        .map(|pair| pair[1].minutes() - pair[0].minutes())
        .collect();
    let interval_sum: u32 = intervals.iter().sum();

    let count = u32::try_from(arrivals.len() - 1).unwrap_or(u32::MAX);
    let mean: f64 = f64::from(interval_sum) / f64::from(count);

    let squared_sum: f64 = intervals
        .iter()
        .map(|&i| f64::from(i) - mean)
        .map(|diff| diff * diff)
        .sum();

    let variance = squared_sum / f64::from(count);

    Some(variance.sqrt())
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineId, LocationId};

    fn time(s: &str) -> TimeOfDay {
        s.parse().unwrap()
    }

    fn times(list: &[&str]) -> Vec<TimeOfDay> {
        list.iter().map(|s| time(s)).collect()
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

    fn assert_close(actual: Option<f64>, expected: f64) {
        let actual = actual.expect("pacing should be Some");
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn pacing_needs_two_arrivals() {
        assert_eq!(pacing(&[]), None);
        assert_eq!(pacing(&times(&["18:00"])), None);
    }

    #[test]
    fn even_arrivals_have_zero_pacing() {
        assert_close(pacing(&times(&["18:00", "18:10", "18:20", "18:30"])), 0.0);
    }

    // intervals 5 and 15: mean 10, variance (25 + 25) / 2 = 25, sqrt 5
    #[test]
    fn uneven_arrivals_give_standard_deviation() {
        assert_close(pacing(&times(&["18:00", "18:05", "18:20"])), 5.0);
    }

    #[test]
    fn no_visits_gives_no_stats() {
        assert_eq!(BarStats::from_visits(&[], Minutes::new(10)), None);
    }

    #[test]
    fn single_visit_has_no_pacing() {
        let stats = BarStats::from_visits(&[visit("18:00", "18:40")], Minutes::new(10)).unwrap();

        assert_eq!(stats.lines_count, 1);
        assert_eq!(stats.pacing, None);
    }

    // intervals 10 and 60: mean 35, variance (625 + 625) / 2 = 625, sqrt 25
    #[test]
    fn stats_of_a_bar_with_one_gap() {
        let visits = [
            visit("18:00", "18:40"),
            visit("18:10", "18:50"),
            visit("19:10", "19:50"),
        ];

        let stats = BarStats::from_visits(&visits, Minutes::new(10)).unwrap();

        assert_eq!(stats.lines_count, 3);
        assert_eq!(stats.first_arrival, time("18:00"));
        assert_eq!(stats.last_departure, time("19:50"));
        assert_eq!(stats.gaps.len(), 1);
        assert_eq!(stats.longest_gap, Some(Minutes::new(20)));
        assert_close(stats.pacing, 25.0);
    }

    #[test]
    fn last_departure_can_come_from_an_earlier_visit() {
        let visits = [visit("18:00", "20:00"), visit("18:30", "19:10")];

        let stats = BarStats::from_visits(&visits, Minutes::new(10)).unwrap();

        assert_eq!(stats.last_departure, time("20:00"));
        assert_eq!(stats.longest_gap, None);
    }
}
