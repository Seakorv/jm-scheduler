mod evaluate;
mod ids;
mod load;
mod model;
mod project;
mod rounding;
mod rules;
mod schedule;
mod settings;
mod stats;
mod time;
mod travel;
mod validate;

pub use crate::evaluate::{EditCheck, Evaluation, check_edit, evaluate};
pub use crate::ids::{AreaId, LineId, LocationId};
pub use crate::load::{LoadError, load_project};
pub use crate::model::{Area, Coordinates, Line, Location, LocationKind, TravelTime, Visit};
pub use crate::project::Project;
pub use crate::rounding::round_up;
pub use crate::rules::{
    Segment, Severity, Violation, ViolationId, ViolationKind, bar_gap_violations, bar_gaps,
    bar_segments, check_capacity, check_duplicate_bar, check_duration, check_incomplete_line,
    check_same_area, check_stay_times, check_stop_count,
};
pub use crate::schedule::{
    ComputedLine, ComputedVisit, IncompleteReason, LineStatus, TimeLeft, bar_timelines,
    compute_all, compute_line,
};
pub use crate::settings::{Language, Settings, TooManyLinesThresholds};
pub use crate::stats::BarStats;
pub use crate::time::{Minutes, ParseTimeError, TimeOfDay, ValidateTimeError};
pub use crate::travel::TravelTable;
pub use crate::validate::{ValidationIssue, validate_project};
