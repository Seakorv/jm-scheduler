mod capacity;
mod duration;
mod gaps;
mod occupancy;
mod stay;
mod structure;
mod violation;

pub use capacity::check_capacity;
pub use duration::check_duration;
pub use gaps::{bar_gap_violations, bar_gaps};
pub use occupancy::{Segment, bar_segments};
pub use stay::check_stay_times;
pub use structure::{
    check_duplicate_bar, check_incomplete_line, check_same_area, check_stop_count,
};
pub use violation::{Severity, Violation, ViolationId, ViolationKind};
