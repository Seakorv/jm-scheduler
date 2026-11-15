use jmetro_core::{
    Candidate, EditCheck, Evaluation, LineId, LoadError, Project, candidates, check_edit, evaluate,
    validate_project,
};
use serde::Deserialize;

use crate::error::{ApiError, AppJson};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckEditRequest {
    before: Project,
    after: Project,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidatesRequest {
    project: Project,
    line_id: LineId,
    visit_index: usize,
}

fn validate(project: &Project) -> Result<(), ApiError> {
    let issues = validate_project(project);
    if !issues.is_empty() {
        return Err(LoadError::Invalid(issues).into());
    }
    Ok(())
}

pub async fn evaluate_handler(
    AppJson(project): AppJson<Project>,
) -> Result<AppJson<Evaluation>, ApiError> {
    validate(&project)?;
    Ok(AppJson(evaluate(&project)))
}

pub async fn check_edit_handler(
    AppJson(request): AppJson<CheckEditRequest>,
) -> Result<AppJson<EditCheck>, ApiError> {
    validate(&request.before)?;
    validate(&request.after)?;
    Ok(AppJson(check_edit(&request.before, &request.after)))
}

pub async fn candidates_handler(
    AppJson(request): AppJson<CandidatesRequest>,
) -> Result<AppJson<Vec<Candidate>>, ApiError> {
    validate(&request.project)?;

    let result = candidates(&request.project, request.line_id, request.visit_index)
        .ok_or_else(|| ApiError::BadRequest(String::from("unknown line or visit index")))?;

    Ok(AppJson(result))
}
