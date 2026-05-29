use std::sync::Arc;

use tauri::State;

use crate::presentation::state::AppState;

fn map_err(e: crate::domain::errors::DomainError) -> String {
    e.to_user_message()
}

#[tauri::command]
pub fn tags_list(state: State<'_, Arc<AppState>>) -> Result<Vec<String>, String> {
    state
        .with_unlocked(|conn, session| {
            state
                .tag_service
                .list_all_tags(conn, &Some(session.clone()))
        })
        .map_err(map_err)
}
