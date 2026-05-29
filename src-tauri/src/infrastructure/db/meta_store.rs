use rusqlite::Connection;

use crate::domain::errors::{DomainError, DomainResult};

pub const KEY_PASSWORD_HASH: &str = "password_hash";
pub const KEY_SALT: &str = "salt";
pub const KEY_LOCK_TIMEOUT: &str = "lock_timeout_secs";

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
