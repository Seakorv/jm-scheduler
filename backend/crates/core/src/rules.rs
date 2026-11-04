mod duration;
mod violation;

pub use duration::check_duration;
pub use violation::{Severity, Violation, ViolationId, ViolationKind};
