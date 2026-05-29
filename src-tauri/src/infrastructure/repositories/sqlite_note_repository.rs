use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::entities::EncryptedNoteRecord;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::NoteRepository;

pub struct SqliteNoteRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteNoteRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<EncryptedNoteRecord> {
        let id: String = row.get(0)?;
        let entry_date: String = row.get(1)?;
        let title_enc: Vec<u8> = row.get(2)?;
        let content_enc: Vec<u8> = row.get(3)?;
        let tags_enc: Vec<u8> = row.get(4)?;
        let is_favorite: i32 = row.get(5)?;
        let created_at: String = row.get(6)?;
        let updated_at: String = row.get(7)?;

        Ok(EncryptedNoteRecord {
            id: Uuid::parse_str(&id).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
            })?,
            entry_date: NaiveDate::parse_from_str(&entry_date, "%Y-%m-%d").map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e))
            })?,
            title_enc,
            content_enc,
            tags_enc,
            is_favorite: is_favorite != 0,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
                })?,
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
                })?,
        })
    }

    fn collect_rows(
        &self,
        mut stmt: rusqlite::Statement<'_>,
        params: impl rusqlite::Params,
    ) -> DomainResult<Vec<EncryptedNoteRecord>> {
        let rows = stmt
            .query_map(params, Self::row_to_record)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(|e| DomainError::Storage(e.to_string()))?);
        }
        Ok(records)
    }
}

const SELECT_COLS: &str =
    "id, entry_date, title_enc, content_enc, tags_enc, is_favorite, created_at, updated_at";

impl NoteRepository for SqliteNoteRepository<'_> {
    fn create(&self, record: &EncryptedNoteRecord) -> DomainResult<()> {
        self.conn
            .execute(
                "INSERT INTO notes (id, entry_date, title_enc, content_enc, tags_enc, is_favorite, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    record.id.to_string(),
                    record.entry_date.format("%Y-%m-%d").to_string(),
                    record.title_enc,
                    record.content_enc,
                    record.tags_enc,
                    record.is_favorite as i32,
                    record.created_at.to_rfc3339(),
                    record.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn update(&self, record: &EncryptedNoteRecord) -> DomainResult<()> {
        let updated = self
            .conn
            .execute(
                "UPDATE notes SET entry_date = ?2, title_enc = ?3, content_enc = ?4, tags_enc = ?5,
                 is_favorite = ?6, updated_at = ?7 WHERE id = ?1",
                params![
                    record.id.to_string(),
                    record.entry_date.format("%Y-%m-%d").to_string(),
                    record.title_enc,
                    record.content_enc,
                    record.tags_enc,
                    record.is_favorite as i32,
                    record.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if updated == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> DomainResult<()> {
        let deleted = self
            .conn
            .execute("DELETE FROM notes WHERE id = ?1", [id])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if deleted == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> DomainResult<Option<EncryptedNoteRecord>> {
        let sql = format!("SELECT {SELECT_COLS} FROM notes WHERE id = ?1");
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        stmt.query_row([id], Self::row_to_record)
            .optional()
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_by_date(&self, date: NaiveDate) -> DomainResult<Option<EncryptedNoteRecord>> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let sql = format!("SELECT {SELECT_COLS} FROM notes WHERE entry_date = ?1 LIMIT 1");
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        stmt.query_row([date_str], Self::row_to_record)
            .optional()
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn list_by_date_range(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> DomainResult<Vec<EncryptedNoteRecord>> {
        match (from, to) {
            (Some(f), Some(t)) => {
                let fs = f.format("%Y-%m-%d").to_string();
                let ts = t.format("%Y-%m-%d").to_string();
                let sql = format!(
                    "SELECT {SELECT_COLS} FROM notes WHERE entry_date >= ?1 AND entry_date <= ?2 ORDER BY entry_date DESC"
                );
                let stmt = self
                    .conn
                    .prepare(&sql)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                self.collect_rows(stmt, params![fs, ts])
            }
            (Some(f), None) => {
                let fs = f.format("%Y-%m-%d").to_string();
                let sql = format!(
                    "SELECT {SELECT_COLS} FROM notes WHERE entry_date >= ?1 ORDER BY entry_date DESC"
                );
                let stmt = self
                    .conn
                    .prepare(&sql)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                self.collect_rows(stmt, params![fs])
            }
            (None, Some(t)) => {
                let ts = t.format("%Y-%m-%d").to_string();
                let sql = format!(
                    "SELECT {SELECT_COLS} FROM notes WHERE entry_date <= ?1 ORDER BY entry_date DESC"
                );
                let stmt = self
                    .conn
                    .prepare(&sql)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                self.collect_rows(stmt, params![ts])
            }
            (None, None) => self.list_all(),
        }
    }

    fn list_favorites(&self) -> DomainResult<Vec<EncryptedNoteRecord>> {
        let sql = format!(
            "SELECT {SELECT_COLS} FROM notes WHERE is_favorite = 1 ORDER BY entry_date DESC"
        );
        let stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        self.collect_rows(stmt, [])
    }

    fn list_future(&self, after: NaiveDate) -> DomainResult<Vec<EncryptedNoteRecord>> {
        let ds = after.format("%Y-%m-%d").to_string();
        let sql = format!(
            "SELECT {SELECT_COLS} FROM notes WHERE entry_date > ?1 ORDER BY entry_date ASC"
        );
        let stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        self.collect_rows(stmt, params![ds])
    }

    fn list_for_month(&self, year: i32, month: u32) -> DomainResult<Vec<EncryptedNoteRecord>> {
        let from = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| DomainError::Validation("Invalid month".into()))?;
        let to = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .and_then(|d| d.pred_opt())
        .ok_or_else(|| DomainError::Validation("Invalid month".into()))?;
        self.list_by_date_range(Some(from), Some(to))
    }

    fn list_all(&self) -> DomainResult<Vec<EncryptedNoteRecord>> {
        let sql = format!("SELECT {SELECT_COLS} FROM notes ORDER BY entry_date DESC");
        let stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        self.collect_rows(stmt, [])
    }
}
