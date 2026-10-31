use uuid::Uuid;

/// Struct for location ID:s (Bars etc.), uuid v7
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocationId(Uuid);

impl LocationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn id(self) -> Uuid {
        self.0
    }
}

impl Default for LocationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Struct for Line ID:s (bus line etc.). uuid v7
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineId(Uuid);

impl LineId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn id(self) -> Uuid {
        self.0
    }
}

impl Default for LineId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_uuids_location() {
        let result1 = LocationId::new();
        let result2 = LocationId::new();

        assert_ne!(result1, result2);
    }

    #[test]
    fn different_uuids_line() {
        let result1 = LineId::new();
        let result2 = LineId::new();

        assert_ne!(result1, result2);
    }

    #[test]
    fn copy_still_same_location() {
        let id = LocationId::new();
        let copy = id;

        assert_eq!(copy, id);
    }

    #[test]
    fn copy_still_same_line() {
        let id = LineId::new();
        let copy = id;

        assert_eq!(copy, id);
    }
}
