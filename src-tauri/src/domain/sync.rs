use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const SYNC_BUNDLE_VERSION: u32 = 1;
pub const SYNC_DRIVE_FILENAME: &str = "locked-calendar-sync.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncTombstone {
    pub id: String,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncNoteRecord {
    pub id: String,
    pub entry_date: String,
    pub title_enc: String,
    pub content_enc: String,
    pub tags_enc: String,
    pub is_favorite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncBundle {
    pub version: u32,
    pub vault_id: String,
    pub kdf_salt: String,
    pub notes: Vec<SyncNoteRecord>,
    pub deleted_ids: Vec<SyncTombstone>,
    pub synced_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncMergeResult {
    pub notes_applied: u32,
    pub notes_kept_local: u32,
    pub tombstones_applied: u32,
    pub notes_deleted: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncStatusResponse {
    pub connected: bool,
    pub last_sync_at: Option<String>,
    pub in_progress: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncNowResponse {
    pub merged: SyncMergeResult,
    pub pushed: bool,
    pub last_sync_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncConnectResponse {
    pub connected: bool,
}
