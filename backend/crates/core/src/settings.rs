use crate::Minutes;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Fi,
    En,
    Sv,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub line_duration: Minutes,
    pub stops_per_line: u32,
    pub default_stay: Minutes,
    pub stay_min_soft: Minutes,
    pub stay_max_soft: Minutes,
    pub rounding_step: Minutes,
    pub departure_step: Minutes,
    pub bar_gap_time: Minutes,
    pub bar_thresholds: TooManyLinesThresholds,
    pub same_area_min_travel: Minutes,
    pub language: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            line_duration: Minutes::new(300),
            stops_per_line: 5,
            default_stay: Minutes::new(40),
            stay_min_soft: Minutes::new(25),
            stay_max_soft: Minutes::new(50),
            rounding_step: Minutes::new(5),
            departure_step: Minutes::new(10),
            bar_gap_time: Minutes::new(10),
            same_area_min_travel: Minutes::new(10),
            bar_thresholds: TooManyLinesThresholds::default(),
            language: Language::Fi,
        }
    }
}

/// Defining when the bar warns when there is lines over the capacity. Leeway gives some leeway to simultaneous lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TooManyLinesThresholds {
    pub minor: u32,
    pub warning: u32,
    pub block: u32,
    pub simultaneous_leeway: Minutes,
}

impl Default for TooManyLinesThresholds {
    fn default() -> Self {
        Self {
            minor: 5,
            warning: 10,
            block: 50,
            simultaneous_leeway: Minutes::new(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let result = Settings::default();
        assert_eq!(result.stops_per_line, 5);
    }
}
