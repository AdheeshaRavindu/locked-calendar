use rusqlite::Connection;

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::sync::SyncTombstone;

pub const KEY_PASSWORD_HASH: &str = "password_hash";
pub const KEY_SALT: &str = "salt";
pub const KEY_LOCK_TIMEOUT: &str = "lock_timeout_secs";
pub const KEY_VAULT_ID: &str = "vault_id";
pub const KEY_SYNC_TOMBSTONES: &str = "sync_tombstones";
pub const KEY_SYNC_DRIVE_FILE_ID: &str = "sync_drive_file_id";
pub const KEY_SYNC_DRIVE_ETAG: &str = "sync_drive_etag";
pub const KEY_SYNC_REFRESH_TOKEN_ENC: &str = "sync_google_refresh_token_enc";
pub const KEY_SYNC_LAST_SYNC_AT: &str = "sync_last_sync_at";

pub const DEFAULT_LOCK_TIMEOUT_SECS: u64 = 600;

pub struct MetaStore<'a> {
    conn: &'a Connection,
}

impl<'a> MetaStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get(&self, key: &str) -> DomainResult<Option<Vec<u8>>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM app_meta WHERE key = ?1")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let result = stmt
            .query_row([key], |row| row.get::<_, Vec<u8>>(0))
            .optional()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(result)
    }

    pub fn set(&self, key: &str, value: &[u8]) -> DomainResult<()> {
        self.conn
            .execute(
                "INSERT INTO app_meta (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                (key, value),
            )
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> DomainResult<()> {
        self.conn
            .execute("DELETE FROM app_meta WHERE key = ?1", [key])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn is_initialized(&self) -> DomainResult<bool> {
        Ok(self.get(KEY_PASSWORD_HASH)?.is_some())
    }

    pub fn get_lock_timeout_secs(&self) -> DomainResult<u64> {
        match self.get(KEY_LOCK_TIMEOUT)? {
            Some(bytes) => {
                let s = String::from_utf8(bytes).map_err(|e| DomainError::Storage(e.to_string()))?;
                s.parse::<u64>()
                    .map_err(|e| DomainError::Storage(e.to_string()))
            }
            None => Ok(DEFAULT_LOCK_TIMEOUT_SECS),
        }
    }

    pub fn set_lock_timeout_secs(&self, secs: u64) -> DomainResult<()> {
        self.set(KEY_LOCK_TIMEOUT, secs.to_string().as_bytes())
    }

    pub fn get_kdf_salt(&self) -> DomainResult<Option<Vec<u8>>> {
        self.get(KEY_SALT)
    }

    pub fn set_kdf_salt(&self, salt: &[u8]) -> DomainResult<()> {
        self.set(KEY_SALT, salt)
    }

    pub fn get_vault_id(&self) -> DomainResult<Option<String>> {
        match self.get(KEY_VAULT_ID)? {
            Some(bytes) => Ok(Some(
                String::from_utf8(bytes).map_err(|e| DomainError::Storage(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    pub fn set_vault_id(&self, vault_id: &str) -> DomainResult<()> {
        self.set(KEY_VAULT_ID, vault_id.as_bytes())
    }

    pub fn ensure_vault_id(&self) -> DomainResult<String> {
        if let Some(id) = self.get_vault_id()? {
            return Ok(id);
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.set_vault_id(&id)?;
        Ok(id)
    }

    pub fn get_tombstones(&self) -> DomainResult<Vec<SyncTombstone>> {
        match self.get(KEY_SYNC_TOMBSTONES)? {
            Some(bytes) => {
                let json = String::from_utf8(bytes).map_err(|e| DomainError::Storage(e.to_string()))?;
                serde_json::from_str(&json).map_err(|e| DomainError::Storage(e.to_string()))
            }
            None => Ok(Vec::new()),
        }
    }

    pub fn set_tombstones(&self, tombstones: &[SyncTombstone]) -> DomainResult<()> {
        let json = serde_json::to_string(tombstones)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        self.set(KEY_SYNC_TOMBSTONES, json.as_bytes())
    }

    pub fn is_sync_connected(&self) -> DomainResult<bool> {
        Ok(self.get(KEY_SYNC_REFRESH_TOKEN_ENC)?.is_some()
            && self.get(KEY_SYNC_DRIVE_FILE_ID)?.is_some())
    }

    pub fn get_sync_drive_file_id(&self) -> DomainResult<Option<String>> {
        match self.get(KEY_SYNC_DRIVE_FILE_ID)? {
            Some(bytes) => Ok(Some(
                String::from_utf8(bytes).map_err(|e| DomainError::Storage(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    pub fn set_sync_drive_file_id(&self, file_id: &str) -> DomainResult<()> {
        self.set(KEY_SYNC_DRIVE_FILE_ID, file_id.as_bytes())
    }

    pub fn get_sync_drive_etag(&self) -> DomainResult<Option<String>> {
        match self.get(KEY_SYNC_DRIVE_ETAG)? {
            Some(bytes) => Ok(Some(
                String::from_utf8(bytes).map_err(|e| DomainError::Storage(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    pub fn set_sync_drive_etag(&self, etag: &str) -> DomainResult<()> {
        self.set(KEY_SYNC_DRIVE_ETAG, etag.as_bytes())
    }

    pub fn get_sync_last_sync_at(&self) -> DomainResult<Option<String>> {
        match self.get(KEY_SYNC_LAST_SYNC_AT)? {
            Some(bytes) => Ok(Some(
                String::from_utf8(bytes).map_err(|e| DomainError::Storage(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    pub fn set_sync_last_sync_at(&self, value: &str) -> DomainResult<()> {
        self.set(KEY_SYNC_LAST_SYNC_AT, value.as_bytes())
    }

    pub fn clear_sync_credentials(&self) -> DomainResult<()> {
        self.delete(KEY_SYNC_REFRESH_TOKEN_ENC)?;
        self.delete(KEY_SYNC_DRIVE_FILE_ID)?;
        self.delete(KEY_SYNC_DRIVE_ETAG)?;
        self.delete(KEY_SYNC_LAST_SYNC_AT)?;
        Ok(())
    }
}

trait OptionalResult {
    type Item;
    fn optional(self) -> rusqlite::Result<Option<Self::Item>>;
}

impl<T> OptionalResult for rusqlite::Result<T> {
    type Item = T;

    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
