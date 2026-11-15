use jmetro_core::{Evaluation, LoadError, Project, evaluate, validate_project};

use crate::error::{ApiError, AppJson};

pub async fn evaluate_handler(
    AppJson(project): AppJson<Project>,
) -> Result<AppJson<Evaluation>, ApiError> {
    let issues = validate_project(&project);
    if !issues.is_empty() {
        return Err(LoadError::Invalid(issues).into());
    }
    Ok(AppJson(evaluate(&project)))
}
