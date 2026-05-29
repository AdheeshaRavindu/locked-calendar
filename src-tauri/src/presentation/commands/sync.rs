use std::sync::Arc;

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::domain::errors::DomainError;
use crate::domain::sync::{SyncConnectResponse, SyncNowResponse, SyncStatusResponse};
use crate::infrastructure::db::meta_store::MetaStore;
use crate::infrastructure::sync::google_drive_provider::GoogleDriveProvider;
use crate::infrastructure::sync::google_oauth::run_oauth_flow;
use crate::presentation::state::AppState;

fn map_err(e: DomainError) -> String {
    e.to_user_message()
}

#[tauri::command]
pub async fn sync_connect(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SyncConnectResponse, String> {
    state.check_idle_lock().map_err(map_err)?;
    let session = state
        .session
        .lock()
        .clone()
        .ok_or_else(|| map_err(DomainError::Locked))?;

    let tokens = run_oauth_flow(|url| {
        app.opener()
            .open_url(url, None::<&str>)
            .map_err(|e| DomainError::Sync(format!("Could not open browser: {e}")))
    })
    .await
    .map_err(map_err)?;

    {
        let conn = state.conn.lock();
        GoogleDriveProvider::store_refresh_token(&conn, &session, &state.crypto, &tokens.refresh_token)
            .map_err(map_err)?;
    }

    let refresh_token = tokens.refresh_token.clone();
    let (file_id, etag) = {
        let conn = state.conn.lock();
        let file_id = MetaStore::new(&conn).get_sync_drive_file_id().map_err(map_err)?;
        let etag = MetaStore::new(&conn).get_sync_drive_etag().map_err(map_err)?;
        (file_id, etag)
    };

    let client_id = crate::infrastructure::sync::google_oauth::google_client_id().map_err(map_err)?;
    let http = reqwest::Client::new();
    let (file_id, etag) = GoogleDriveProvider::ensure_drive_file(
        &client_id,
        &http,
        &refresh_token,
        tokens.access_token,
        file_id,
        etag,
    )
    .await
    .map_err(map_err)?;

    {
        let conn = state.conn.lock();
        MetaStore::new(&conn).set_sync_drive_file_id(&file_id).map_err(map_err)?;
        if let Some(tag) = etag {
            MetaStore::new(&conn).set_sync_drive_etag(&tag).map_err(map_err)?;
        }
    }

    state.emit_sync_status(&app);
    Ok(SyncConnectResponse { connected: true })
}

#[tauri::command]
pub fn sync_disconnect(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state
        .with_unlocked(|conn, _session| MetaStore::new(conn).clear_sync_credentials())
        .map_err(map_err)?;
    *state.sync_last_error.lock() = None;
    state.emit_sync_status(&app);
    Ok(())
}

#[tauri::command]
pub async fn sync_now(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<SyncNowResponse, String> {
    state.execute_sync(&app).await.map_err(map_err)
}

#[tauri::command]
pub fn sync_status(state: State<'_, Arc<AppState>>) -> Result<SyncStatusResponse, String> {
    state.sync_status().map_err(map_err)
}

pub fn trigger_debounced_sync(app: AppHandle, state: &State<'_, Arc<AppState>>) {
    state.schedule_debounced_sync(app);
}
