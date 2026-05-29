use std::time::{Duration, Instant};

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::application::ports::CryptoProvider;
use crate::application::services::{AuthService, NoteService, SearchService};
use crate::application::services::auth_service::SessionKey;
use crate::domain::errors::{DomainError, DomainResult};
use crate::infrastructure::crypto::AesGcmCryptoProvider;
use crate::infrastructure::db::meta_store::MetaStore;
use std::sync::Arc;

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub session: Mutex<Option<SessionKey>>,
    pub last_activity: Mutex<Instant>,
    pub lock_timeout: Mutex<Duration>,
    pub auth_service: AuthService,
    pub note_service: NoteService,
    pub search_service: SearchService,
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
            search_service: SearchService::new(crypto),
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
}
