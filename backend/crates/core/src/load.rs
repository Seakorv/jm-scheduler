use serde::Deserialize;
use thiserror::Error;

use crate::project::SCHEMA_VERSION;
use crate::validate::ValidationIssue;
use crate::{Project, validate_project};

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("unsupported file version {0}")]
    UnsupportedVersion(u32),
    #[error("file has incorrect data")]
    Invalid(Vec<ValidationIssue>),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionHeader {
    schema_version: u32,
}

/// Loads the project. Checks the version first, then loads it from json
///
/// # Errors
///
/// - New schema version returns [`LoadError::UnsupportedVersion`]
/// - Invalid JSON returns [`LoadError::InvalidJson`]
pub fn load_project(json: &str) -> Result<Project, LoadError> {
    let header: VersionHeader = serde_json::from_str(json)?;
    if header.schema_version > SCHEMA_VERSION {
        return Err(LoadError::UnsupportedVersion(header.schema_version));
    }
    let project: Project = serde_json::from_str(json)?;
    let issues = validate_project(&project);
    if !issues.is_empty() {
        return Err(LoadError::Invalid(issues));
    }
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../../../../examples/2024.jvms.json");

    #[test]
    fn rejects_newer_schema_version() {
        let result = load_project(r#"{ "schemaVersion": 67 }"#);

        assert!(matches!(result, Err(LoadError::UnsupportedVersion(67))));
    }

    #[test]
    fn example_file_loads() {
        let project = load_project(EXAMPLE).unwrap();

        assert_eq!(project.lines.len(), 2);
    }
}
