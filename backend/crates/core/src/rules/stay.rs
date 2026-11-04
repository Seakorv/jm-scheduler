use crate::{Line, Settings, Severity, Violation, ViolationKind};

/// Checks visit times for lines and if they stay within ideal boundaries. Returns violations if they are too long or short
pub fn check_stay_times(line: &Line, settings: &Settings) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (i, visit) in line.visits.iter().enumerate() {
        if visit.stay_time < settings.stay_min_soft {
            violations.push(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::StayTooShort {
                    line_id: line.id,
                    visit_index: i,
                    stay_time: visit.stay_time,
                },
            });
        }
        if visit.stay_time > settings.stay_max_soft {
            violations.push(Violation {
                severity: Severity::Warning,
                kind: ViolationKind::StayTooLong {
                    line_id: line.id,
                    visit_index: i,
                    stay_time: visit.stay_time,
                },
            });
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LocationId, Minutes, TimeOfDay, Visit};

    #[test]
    fn stays_outside_soft_limits_warn() {
        // default soft limits are 25 and 50
        let mut line = Line::new(1, None, false, TimeOfDay::new(960).unwrap(), None);
        for stay in [20, 25, 50, 55] {
            line.visits.push(Visit {
                location_id: LocationId::new(),
                stay_time: Minutes::new(stay),
            });
        }

        let violations = check_stay_times(&line, &Settings::default());

        assert_eq!(
            violations,
            vec![
                Violation {
                    severity: Severity::Warning,
                    kind: ViolationKind::StayTooShort {
                        line_id: line.id,
                        visit_index: 0,
                        stay_time: Minutes::new(20),
                    },
                },
                Violation {
                    severity: Severity::Warning,
                    kind: ViolationKind::StayTooLong {
                        line_id: line.id,
                        visit_index: 3,
                        stay_time: Minutes::new(55),
                    },
                },
            ]
        );
    }
}
