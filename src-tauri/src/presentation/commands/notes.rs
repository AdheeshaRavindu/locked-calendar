use chrono::{Local, NaiveDate};
use tauri::State;

use crate::application::dto::{NoteResponse, SaveNoteRequest};
use crate::domain::entities::DayMarker;
use crate::presentation::state::AppState;

fn map_err(e: crate::domain::errors::DomainError) -> String {
    e.to_user_message()
}

fn parse_date(date: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| "Invalid date format. Use YYYY-MM-DD.".into())
}

#[tauri::command]
pub fn notes_get_today(state: State<'_, AppState>) -> Result<NoteResponse, String> {
    let today = Local::now().date_naive();
    state
        .with_unlocked(|conn, session| {
            state
                .note_service
                .get_or_create_for_date(conn, &Some(session.clone()), today)
        })
        .map_err(map_err)
}

#[tauri::command]
pub fn notes_get_by_date(state: State<'_, AppState>, date: String) -> Result<Option<NoteResponse>, String> {
    let entry_date = parse_date(&date)?;
    state
        .with_unlocked(|conn, session| {
            state
                .note_service
                .get_by_date(conn, &Some(session.clone()), entry_date)
        })
        .map_err(map_err)
}

#[tauri::command]
pub fn notes_get_or_create(state: State<'_, AppState>, date: String) -> Result<NoteResponse, String> {
    let entry_date = parse_date(&date)?;
    state
        .with_unlocked(|conn, session| {
            state
                .note_service
                .get_or_create_for_date(conn, &Some(session.clone()), entry_date)
        })
        .map_err(map_err)
}

#[tauri::command]
pub fn notes_save(state: State<'_, AppState>, payload: SaveNoteRequest) -> Result<NoteResponse, String> {
    let entry_date = parse_date(&payload.entry_date)?;
    state
        .with_unlocked(|conn, session| {
            state.note_service.save(
                conn,
                &Some(session.clone()),
                payload.id,
                entry_date,
                payload.title,
                payload.content,
                payload.tags,
                payload.is_favorite,
            )
        })
        .map_err(map_err)
}

#[tauri::command]
pub fn notes_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .with_unlocked(|conn, session| {
            state
                .note_service
                .delete(conn, &Some(session.clone()), &id)
        })
        .map_err(map_err)
}

#[tauri::command]
pub fn notes_toggle_favorite(state: State<'_, AppState>, id: String) -> Result<NoteResponse, String> {
    state
        .with_unlocked(|conn, session| {
            state
                .note_service
                .toggle_favorite(conn, &Some(session.clone()), &id)
        })
        .map_err(map_err)
}

#[tauri::command]
pub fn notes_list_month(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<Vec<DayMarker>, String> {
    state
        .with_unlocked(|conn, session| {
            state
                .note_service
                .list_month_markers(conn, &Some(session.clone()), year, month)
        })
        .map_err(map_err)
}
