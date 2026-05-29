use crate::domain::errors::DomainResult;
use crate::domain::sync::SyncBundle;
#[derive(Debug, Clone)]
pub struct RemoteSyncPayload {
    pub bundle: SyncBundle,
    pub etag: Option<String>,
}

#[async_trait::async_trait]
pub trait SyncProvider: Send + Sync {
    async fn pull(&self) -> DomainResult<Option<RemoteSyncPayload>>;
    async fn push(&self, bundle: &SyncBundle, etag: Option<&str>) -> DomainResult<String>;
}
