use chrono::NaiveDate;
use tauri::State;

use crate::application::dto::SearchNotesRequest;
use crate::domain::entities::NoteSummary;
use crate::presentation::state::AppState;

fn map_err(e: crate::domain::errors::DomainError) -> String {
    e.to_user_message()
}

fn parse_optional_date(value: &Option<String>) -> Result<Option<NaiveDate>, String> {
    match value {
        Some(d) if !d.is_empty() => Ok(Some(
            NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|_| "Invalid date format. Use YYYY-MM-DD.".to_string())?,
        )),
        _ => Ok(None),
    }
}

#[tauri::command]
pub fn search_notes(
    state: State<'_, AppState>,
    payload: SearchNotesRequest,
) -> Result<Vec<NoteSummary>, String> {
    let date_from = parse_optional_date(&payload.date_from)?;
    let date_to = parse_optional_date(&payload.date_to)?;
    let query = payload.query.filter(|q| !q.trim().is_empty());

    state
        .with_unlocked(|conn, session| {
            state.search_service.search(
                conn,
                &Some(session.clone()),
                query.as_deref(),
                date_from,
                date_to,
                &payload.tags,
                payload.favorites_only,
                payload.future_only,
            )
        })
        .map_err(map_err)
}
