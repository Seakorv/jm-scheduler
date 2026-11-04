mod duration;
mod stay;
mod structure;
mod violation;

pub use duration::check_duration;
pub use stay::check_stay_times;
pub use structure::check_stop_count;
pub use violation::{Severity, Violation, ViolationId, ViolationKind};
