use chrono::NaiveDate;
use tauri::State;

use crate::application::dto::{OnThisDayEntry, TimelineGroup};
use crate::presentation::state::AppState;

fn map_err(e: crate::domain::errors::DomainError) -> String {
    e.to_user_message()
}

fn parse_date(date: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| "Invalid date format. Use YYYY-MM-DD.".into())
}

#[tauri::command]
pub fn timeline_list(state: State<'_, AppState>) -> Result<Vec<TimelineGroup>, String> {
    state
        .with_unlocked(|conn, session| {
            state
                .timeline_service
                .list_grouped(conn, &Some(session.clone()))
        })
        .map_err(map_err)
}

#[tauri::command]
pub fn notes_on_this_day(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<OnThisDayEntry>, String> {
    let entry_date = parse_date(&date)?;
    state
        .with_unlocked(|conn, session| {
            state.on_this_day_service.get_memories(
                conn,
                &Some(session.clone()),
                entry_date,
            )
        })
        .map_err(map_err)
}
