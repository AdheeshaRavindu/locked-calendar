use std::sync::{Arc, Mutex};

use crate::application::ports::{RemoteSyncPayload, SyncProvider};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::sync::SyncBundle;

#[derive(Default)]
pub struct MockSyncProvider {
    remote: Mutex<Option<RemoteSyncPayload>>,
    pushed: Mutex<Vec<(SyncBundle, Option<String>)>>,
}

impl MockSyncProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_remote(&self, payload: Option<RemoteSyncPayload>) {
        *self.remote.lock().unwrap() = payload;
    }

    pub fn pushed_bundles(&self) -> Vec<(SyncBundle, Option<String>)> {
        self.pushed.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl SyncProvider for MockSyncProvider {
    async fn pull(&self) -> DomainResult<Option<RemoteSyncPayload>> {
        Ok(self.remote.lock().unwrap().clone())
    }

    async fn push(&self, bundle: &SyncBundle, etag: Option<&str>) -> DomainResult<String> {
        self.pushed
            .lock()
            .unwrap()
            .push((bundle.clone(), etag.map(str::to_string)));
        Ok("mock-etag".into())
    }
}

pub type SharedMockSyncProvider = Arc<MockSyncProvider>;
