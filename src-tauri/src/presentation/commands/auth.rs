use tauri::{AppHandle, Emitter, State};

use crate::application::dto::AuthStatusResponse;
use crate::domain::errors::DomainError;
use crate::infrastructure::db::meta_store::MetaStore;
use crate::presentation::state::AppState;

fn map_err(e: DomainError) -> String {
    e.to_user_message()
}

#[tauri::command]
pub fn auth_is_initialized(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .with_conn(|conn| state.auth_service.is_initialized(conn))
        .map_err(map_err)
}

#[tauri::command]
pub fn auth_status(state: State<'_, AppState>) -> Result<AuthStatusResponse, String> {
    let _ = state.check_idle_lock();
    let initialized = state
        .with_conn(|conn| state.auth_service.is_initialized(conn))
        .map_err(map_err)?;
    Ok(AuthStatusResponse {
        initialized,
        unlocked: state.is_unlocked(),
    })
}

#[tauri::command]
pub fn auth_setup(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
) -> Result<(), String> {
    let session = state
        .with_conn(|conn| state.auth_service.setup_master_password(conn, &password))
        .map_err(map_err)?;
    *state.session.lock() = Some(session);
    state.touch_activity();
    let _ = app.emit("session-unlocked", ());
    Ok(())
}

#[tauri::command]
pub fn auth_unlock(
    app: AppHandle,
    state: State<'_, AppState>,
    password: String,
) -> Result<(), String> {
    let session = state
        .with_conn(|conn| state.auth_service.unlock(conn, &password))
        .map_err(map_err)?;
    *state.session.lock() = Some(session);
    state.touch_activity();
    let _ = app.emit("session-unlocked", ());
    Ok(())
}

#[tauri::command]
pub fn auth_change_password(
    state: State<'_, AppState>,
    old_password: String,
    new_password: String,
) -> Result<(), String> {
    let session = state
        .with_unlocked(|conn, _session| {
            state.auth_service.change_password(
                conn,
                &state.note_service,
                &old_password,
                &new_password,
            )
        })
        .map_err(map_err)?;
    *state.session.lock() = Some(session);
    state.touch_activity();
    Ok(())
}

#[tauri::command]
pub fn auth_lock(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    state.lock();
    let _ = app.emit("session-locked", ());
    Ok(())
}

#[tauri::command]
pub fn auth_touch_session(state: State<'_, AppState>) -> Result<(), String> {
    state.check_idle_lock().map_err(map_err)?;
    if state.is_unlocked() {
        state.touch_activity();
    }
    Ok(())
}

#[tauri::command]
pub fn auth_get_lock_timeout(state: State<'_, AppState>) -> Result<u64, String> {
    state
        .with_conn(|conn| MetaStore::new(conn).get_lock_timeout_secs())
        .map_err(map_err)
}

#[tauri::command]
pub fn auth_set_lock_timeout(state: State<'_, AppState>, seconds: u64) -> Result<(), String> {
    if seconds < 60 {
        return Err("Lock timeout must be at least 60 seconds.".into());
    }
    state
        .with_conn(|conn| MetaStore::new(conn).set_lock_timeout_secs(seconds))
        .map_err(map_err)?;
    *state.lock_timeout.lock() = std::time::Duration::from_secs(seconds);
    Ok(())
}
