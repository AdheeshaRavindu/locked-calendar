use std::sync::Arc;

use tauri::State;

use crate::presentation::state::AppState;

fn map_err(e: crate::domain::errors::DomainError) -> String {
    e.to_user_message()
}

#[tauri::command]
pub fn export_notes_json(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    state
        .with_unlocked(|conn, session| {
            state
                .export_service
                .export_json(conn, &Some(session.clone()))
        })
        .map_err(map_err)
}
