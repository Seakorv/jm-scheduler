use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        /// Struct for ID:s, uuid v7
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn id(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

id_type!(LocationId);
id_type!(LineId);
id_type!(AreaId);

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
    fn different_uuids_area() {
        let result1 = AreaId::new();
        let result2 = AreaId::new();

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

    #[test]
    fn copy_still_same_area() {
        let id = AreaId::new();
        let copy = id;

        assert_eq!(copy, id);
    }
}
