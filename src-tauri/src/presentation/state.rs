use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::Connection;
use tauri::{AppHandle, Emitter};

use crate::application::ports::CryptoProvider;
use crate::application::ports::SyncProvider;
use crate::application::services::{
    AuthService, ExportService, NoteService, OnThisDayService, SearchService, SyncService,
    TagService, TimelineService,
};
use crate::application::services::auth_service::SessionKey;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::sync::{SyncNowResponse, SyncStatusResponse};
use crate::infrastructure::crypto::AesGcmCryptoProvider;
use crate::infrastructure::db::meta_store::MetaStore;
use crate::infrastructure::sync::SyncProviderFactory;

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub session: Mutex<Option<SessionKey>>,
    pub last_activity: Mutex<Instant>,
    pub lock_timeout: Mutex<Duration>,
    pub crypto: Arc<dyn CryptoProvider>,
    pub auth_service: AuthService,
    pub note_service: NoteService,
    pub search_service: SearchService,
    pub timeline_service: TimelineService,
    pub on_this_day_service: OnThisDayService,
    pub tag_service: TagService,
    pub export_service: ExportService,
    pub sync_generation: Mutex<u64>,
    pub sync_in_progress: Mutex<bool>,
    pub sync_last_error: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        let crypto: Arc<dyn CryptoProvider> = Arc::new(AesGcmCryptoProvider::new());
        let lock_secs = MetaStore::new(&conn)
            .get_lock_timeout_secs()
            .unwrap_or(600);
        Self {
            conn: Mutex::new(conn),
            session: Mutex::new(None),
            last_activity: Mutex::new(Instant::now()),
            lock_timeout: Mutex::new(Duration::from_secs(lock_secs)),
            auth_service: AuthService::new(Arc::clone(&crypto)),
            note_service: NoteService::new(Arc::clone(&crypto)),
            search_service: SearchService::new(Arc::clone(&crypto)),
            timeline_service: TimelineService::new(Arc::clone(&crypto)),
            on_this_day_service: OnThisDayService::new(Arc::clone(&crypto)),
            tag_service: TagService::new(Arc::clone(&crypto)),
            export_service: ExportService::new(Arc::clone(&crypto)),
            crypto,
            sync_generation: Mutex::new(0),
            sync_in_progress: Mutex::new(false),
            sync_last_error: Mutex::new(None),
        }
    }

    pub fn touch_activity(&self) {
        *self.last_activity.lock() = Instant::now();
    }

    pub fn is_unlocked(&self) -> bool {
        self.session.lock().is_some()
    }

    pub fn lock(&self) {
        let mut session = self.session.lock();
        *session = None;
    }

    pub fn check_idle_lock(&self) -> DomainResult<()> {
        if self.session.lock().is_none() {
            return Ok(());
        }
        let elapsed = self.last_activity.lock().elapsed();
        let timeout = *self.lock_timeout.lock();
        if elapsed >= timeout {
            self.lock();
            return Err(DomainError::Locked);
        }
        Ok(())
    }

    pub fn with_conn<F, T>(&self, f: F) -> DomainResult<T>
    where
        F: FnOnce(&Connection) -> DomainResult<T>,
    {
        let conn = self.conn.lock();
        f(&conn)
    }

    pub fn with_unlocked<F, T>(&self, f: F) -> DomainResult<T>
    where
        F: FnOnce(&Connection, &SessionKey) -> DomainResult<T>,
    {
        self.check_idle_lock()?;
        let session_guard = self.session.lock();
        let session = session_guard.as_ref().ok_or(DomainError::Locked)?;
        self.touch_activity();
        let conn = self.conn.lock();
        f(&conn, session)
    }

    pub fn sync_status(&self) -> DomainResult<SyncStatusResponse> {
        let conn = self.conn.lock();
        Ok(SyncStatusResponse {
            connected: MetaStore::new(&conn).is_sync_connected()?,
            last_sync_at: MetaStore::new(&conn).get_sync_last_sync_at()?,
            in_progress: *self.sync_in_progress.lock(),
            error: self.sync_last_error.lock().clone(),
        })
    }

    pub fn emit_sync_status(&self, app: &AppHandle) {
        if let Ok(status) = self.sync_status() {
            let _ = app.emit("sync-status-changed", status);
        }
    }

    pub async fn execute_sync(&self, app: &AppHandle) -> DomainResult<SyncNowResponse> {
        if !self.is_unlocked() {
            return Err(DomainError::Locked);
        }
        {
            let mut in_progress = self.sync_in_progress.lock();
            if *in_progress {
                return Err(DomainError::Sync("Sync is already in progress.".into()));
            }
            *in_progress = true;
        }
        *self.sync_last_error.lock() = None;
        self.emit_sync_status(app);

        let result = async {
            let provider = {
                let conn = self.conn.lock();
                if !MetaStore::new(&conn).is_sync_connected()? {
                    return Err(DomainError::Sync("Google Drive is not connected.".into()));
                }
                let session = self
                    .session
                    .lock()
                    .clone()
                    .ok_or(DomainError::Locked)?;
                SyncProviderFactory::google_drive(&conn, &session, &self.crypto)?
            };

            let remote = provider.pull().await?;
            let mut merged = crate::domain::sync::SyncMergeResult::default();
            let mut etag = {
                let conn = self.conn.lock();
                MetaStore::new(&conn).get_sync_drive_etag()?
            };

            if let Some(payload) = remote {
                {
                    let conn = self.conn.lock();
                    merged = SyncService::merge_remote(&conn, &payload.bundle)?;
                }
                etag = payload.etag;
            }

            let bundle = {
                let conn = self.conn.lock();
                SyncService::build_bundle(&conn)?
            };
            let push_result = provider.push(&bundle, etag.as_deref()).await;
            let new_etag = match push_result {
                Ok(tag) => tag,
                Err(DomainError::Sync(msg)) if msg.contains("changed on another device") => {
                    if let Some(payload) = provider.pull().await? {
                        {
                            let conn = self.conn.lock();
                            merged = SyncService::merge_remote(&conn, &payload.bundle)?;
                        }
                        etag = payload.etag;
                    }
                    let bundle = {
                        let conn = self.conn.lock();
                        SyncService::build_bundle(&conn)?
                    };
                    provider.push(&bundle, etag.as_deref()).await?
                }
                Err(err) => return Err(err),
            };

            let synced_at = Utc::now().to_rfc3339();
            {
                let conn = self.conn.lock();
                MetaStore::new(&conn).set_sync_drive_etag(&new_etag)?;
                MetaStore::new(&conn).set_sync_last_sync_at(&synced_at)?;
            }
            Ok(SyncNowResponse {
                merged,
                pushed: true,
                last_sync_at: synced_at,
            })
        }
        .await;

        *self.sync_in_progress.lock() = false;
        match &result {
            Ok(_) => *self.sync_last_error.lock() = None,
            Err(err) => *self.sync_last_error.lock() = Some(err.to_user_message()),
        }
        self.emit_sync_status(app);
        result
    }

    pub fn schedule_debounced_sync(self: &Arc<Self>, app: AppHandle) {
        if !self.is_unlocked() {
            return;
        }
        {
            let conn = self.conn.lock();
            if !MetaStore::new(&conn).is_sync_connected().unwrap_or(false) {
                return;
            }
        }

        let generation = {
            let mut gen = self.sync_generation.lock();
            *gen += 1;
            *gen
        };

        let state = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            if *state.sync_generation.lock() != generation {
                return;
            }
            let _ = state.execute_sync(&app).await;
        });
    }
}
