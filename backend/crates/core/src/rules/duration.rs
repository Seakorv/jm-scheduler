use crate::{ComputedLine, LineStatus, Severity, TimeLeft, Violation, ViolationKind};

/// Checks the duration the line has left to allocate and returns an `Option<Violation>` based if it is over or under or zero.
pub fn check_duration(line: &ComputedLine) -> Option<Violation> {
    match line.status {
        LineStatus::Complete {
            time_left: TimeLeft::Short(minutes),
            ..
        } => Some(Violation {
            severity: Severity::Warning,
            kind: ViolationKind::TimeLeft {
                line_id: line.line_id,
                minutes,
            },
        }),
        LineStatus::Complete {
            time_left: TimeLeft::Over(minutes),
            ..
        } => Some(Violation {
            severity: Severity::Warning,
            kind: ViolationKind::TimeOver {
                line_id: line.line_id,
                minutes,
            },
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IncompleteReason, LineId, Minutes, TimeOfDay};

    fn complete_line(time_left: TimeLeft) -> ComputedLine {
        ComputedLine {
            line_id: LineId::new(),
            visits: vec![],
            status: LineStatus::Complete {
                end_arrival: TimeOfDay::new(1260).unwrap(),
                travel_to_end: Minutes::new(10),
                total: Minutes::new(300),
                time_left,
            },
        }
    }

    #[test]
    fn exactly_on_time_has_no_violation() {
        assert_eq!(check_duration(&complete_line(TimeLeft::Zero)), None);
    }

    #[test]
    fn time_left_warns() {
        let line = complete_line(TimeLeft::Short(Minutes::new(10)));

        assert_eq!(
            check_duration(&line),
            Some(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::TimeLeft {
                    line_id: line.line_id,
                    minutes: Minutes::new(10),
                },
            })
        );
    }

    #[test]
    fn time_over_warns() {
        let line = complete_line(TimeLeft::Over(Minutes::new(5)));

        assert_eq!(
            check_duration(&line),
            Some(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::TimeOver {
                    line_id: line.line_id,
                    minutes: Minutes::new(5),
                },
            })
        );
    }

    #[test]
    fn incomplete_line_has_no_duration_violation() {
        let line = ComputedLine {
            line_id: LineId::new(),
            visits: vec![],
            status: LineStatus::Incomplete {
                from_index: 0,
                reason: IncompleteReason::PastMidnight,
            },
        };

        assert_eq!(check_duration(&line), None);
    }
}
