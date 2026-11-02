use crate::{LineId, LocationId, Minutes, TimeOfDay};
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
