mod duration;
mod stay;
mod violation;

pub use duration::check_duration;
pub use stay::check_stay_times;
pub use violation::{Severity, Violation, ViolationId, ViolationKind};
