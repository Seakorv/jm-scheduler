use std::mem::{Discriminant, discriminant};

use serde::Serialize;

use crate::{AreaId, LineId, LocationId, Minutes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Slight,
    Warning,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ViolationKind {
    TooManyLines {
        line_id: LineId,
        location_id: LocationId,
        visit_index: usize,
        overloaded: Minutes,
        ratio_pct: u32,
    },
    TimeLeft {
        line_id: LineId,
        minutes: Minutes,
    },
    TimeOver {
        line_id: LineId,
        minutes: Minutes,
    },
    StayTooShort {
        line_id: LineId,
        visit_index: usize,
        stay_time: Minutes,
    },
    StayTooLong {
        line_id: LineId,
        visit_index: usize,
        stay_time: Minutes,
    },
    BarGaps {
        location_id: LocationId,
        count: u32,
        longest: Minutes,
    },
    NoVisits {
        line_id: LineId,
    },
    WrongStopCount {
        line_id: LineId,
        count: u32,
        expected: u32,
    },
    DuplicateBar {
        line_id: LineId,
        location_id: LocationId,
    },
    MissingTravelTime {
        line_id: LineId,
        from: LocationId,
        to: LocationId,
    },
    SameAreaInARow {
        line_id: LineId,
        visit_index: usize,
        area_id: AreaId,
    },
    PastMidnight {
        line_id: LineId,
        visit_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    pub severity: Severity,
    pub kind: ViolationKind,
}

impl Violation {
    /// Returns the ID of a violation
    pub fn identity(&self) -> ViolationId {
        let (line_id, location_id, visit_index) = match &self.kind {
            ViolationKind::TooManyLines {
                line_id,
                location_id,
                visit_index,
                ..
            } => (Some(*line_id), Some(*location_id), Some(*visit_index)),
            ViolationKind::TimeLeft { line_id, .. }
            | ViolationKind::TimeOver { line_id, .. }
            | ViolationKind::NoVisits { line_id }
            | ViolationKind::WrongStopCount { line_id, .. } => (Some(*line_id), None, None),
            ViolationKind::StayTooShort {
                line_id,
                visit_index,
                ..
            }
            | ViolationKind::StayTooLong {
                line_id,
                visit_index,
                ..
            }
            | ViolationKind::SameAreaInARow {
                line_id,
                visit_index,
                ..
            }
            | ViolationKind::PastMidnight {
                line_id,
                visit_index,
            } => (Some(*line_id), None, Some(*visit_index)),
            ViolationKind::BarGaps { location_id, .. } => (None, Some(*location_id), None),
            ViolationKind::DuplicateBar {
                line_id,
                location_id,
            } => (Some(*line_id), Some(*location_id), None),
            ViolationKind::MissingTravelTime { line_id, to, .. } => {
                (Some(*line_id), Some(*to), None)
            }
        };
        ViolationId {
            kind: discriminant(&self.kind),
            line_id,
            location_id,
            visit_index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViolationId {
    kind: Discriminant<ViolationKind>,
    line_id: Option<LineId>,
    location_id: Option<LocationId>,
    visit_index: Option<usize>,
}

// Claude generated tests
#[cfg(test)]
mod tests {
    use super::*;

    // The derived order follows the variant order, so reordering them would break this.
    #[test]
    fn severities_are_ordered_slight_warning_block() {
        assert!(Severity::Slight < Severity::Warning);
        assert!(Severity::Warning < Severity::Block);
    }

    #[test]
    fn max_gives_the_worst_severity() {
        let severities = [Severity::Warning, Severity::Block, Severity::Slight];

        assert_eq!(severities.iter().max(), Some(&Severity::Block));
    }

    fn too_many_lines(line_id: LineId, location_id: LocationId, ratio_pct: u32) -> Violation {
        Violation {
            severity: Severity::Block,
            kind: ViolationKind::TooManyLines {
                line_id,
                location_id,
                visit_index: 2,
                overloaded: Minutes::new(30),
                ratio_pct,
            },
        }
    }

    #[test]
    fn violation_json_has_type_tag_and_camel_case_fields() {
        let violation = too_many_lines(LineId::new(), LocationId::new(), 75);

        let json = serde_json::to_value(&violation).unwrap();

        assert_eq!(json["severity"], "block");
        assert_eq!(json["kind"]["type"], "tooManyLines");
        assert_eq!(json["kind"]["visitIndex"], 2);
        assert_eq!(json["kind"]["ratioPct"], 75);
    }

    #[test]
    fn identity_ignores_measurements() {
        let line = LineId::new();
        let bar = LocationId::new();

        assert_eq!(
            too_many_lines(line, bar, 60).identity(),
            too_many_lines(line, bar, 65).identity()
        );
    }

    #[test]
    fn identity_differs_by_bar() {
        let line = LineId::new();

        assert_ne!(
            too_many_lines(line, LocationId::new(), 60).identity(),
            too_many_lines(line, LocationId::new(), 60).identity()
        );
    }

    #[test]
    fn identity_differs_by_kind() {
        let line_id = LineId::new();
        let left = Violation {
            severity: Severity::Warning,
            kind: ViolationKind::TimeLeft {
                line_id,
                minutes: Minutes::new(10),
            },
        };
        let over = Violation {
            severity: Severity::Warning,
            kind: ViolationKind::TimeOver {
                line_id,
                minutes: Minutes::new(10),
            },
        };

        assert_ne!(left.identity(), over.identity());
    }
}
