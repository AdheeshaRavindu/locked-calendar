use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use rusqlite::Connection;
use uuid::Uuid;

use crate::application::dto::NoteResponse;
use crate::application::ports::CryptoProvider;
use crate::application::services::auth_service::SessionKey;
use crate::application::services::SyncService;
use crate::domain::entities::{DayMarker, EncryptedNoteRecord, Note};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::NoteRepository;
use crate::infrastructure::repositories::SqliteNoteRepository;

const MAX_TITLE_LEN: usize = 512;
const MAX_CONTENT_LEN: usize = 1_048_576;

pub struct NoteService {
    crypto: Arc<dyn CryptoProvider>,
}

impl NoteService {
    pub fn new(crypto: Arc<dyn CryptoProvider>) -> Self {
        Self { crypto }
    }

    fn repo<'a>(&self, conn: &'a Connection) -> SqliteNoteRepository<'a> {
        SqliteNoteRepository::new(conn)
    }

    fn require_session<'a>(
        &self,
        session: &'a Option<SessionKey>,
    ) -> DomainResult<&'a [u8; 32]> {
        session
            .as_ref()
            .map(|s| &s.0)
            .ok_or(DomainError::Locked)
    }

    fn validate_note_input(title: &str, content: &str) -> DomainResult<()> {
        if title.len() > MAX_TITLE_LEN {
            return Err(DomainError::Validation("Title is too long.".into()));
        }
        if content.len() > MAX_CONTENT_LEN {
            return Err(DomainError::Validation("Content is too long.".into()));
        }
        Ok(())
    }

    fn validate_mood(mood: Option<u8>) -> DomainResult<()> {
        if let Some(m) = mood {
            if !(1..=5).contains(&m) {
                return Err(DomainError::Validation("Mood must be between 1 and 5.".into()));
            }
        }
        Ok(())
    }

    pub fn is_metadata_only(note: &Note) -> bool {
        note.title.is_empty()
            && note.content.is_empty()
            && note.tags.is_empty()
            && !note.is_favorite
            && !note.is_done
            && note.mood.is_none()
    }

    fn encrypt_record(
        &self,
        note: &Note,
        key: &[u8; 32],
    ) -> DomainResult<EncryptedNoteRecord> {
        let tags_json = serde_json::to_string(&note.tags)
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(EncryptedNoteRecord {
            id: note.id,
            entry_date: note.entry_date,
            title_enc: self.crypto.encrypt(&note.title, key)?,
            content_enc: self.crypto.encrypt(&note.content, key)?,
            tags_enc: self.crypto.encrypt(&tags_json, key)?,
            is_favorite: note.is_favorite,
            is_done: note.is_done,
            mood: note.mood,
            created_at: note.created_at,
            updated_at: note.updated_at,
        })
    }

    fn decrypt_record(
        &self,
        record: &EncryptedNoteRecord,
        key: &[u8; 32],
    ) -> DomainResult<Note> {
        let title = self.crypto.decrypt(&record.title_enc, key)?;
        let content = self.crypto.decrypt(&record.content_enc, key)?;
        let tags_json = self.crypto.decrypt(&record.tags_enc, key)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json)
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(Note {
            id: record.id,
            entry_date: record.entry_date,
            title,
            content,
            tags,
            is_favorite: record.is_favorite,
            is_done: record.is_done,
            mood: record.mood,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    pub fn to_response(note: &Note) -> NoteResponse {
        NoteResponse {
            id: note.id.to_string(),
            entry_date: note.entry_date.format("%Y-%m-%d").to_string(),
            title: note.title.clone(),
            content: note.content.clone(),
            tags: note.tags.clone(),
            is_favorite: note.is_favorite,
            is_done: note.is_done,
            mood: note.mood,
            created_at: note.created_at.to_rfc3339(),
            updated_at: note.updated_at.to_rfc3339(),
        }
    }

    pub fn get_by_date(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
        date: NaiveDate,
    ) -> DomainResult<Option<NoteResponse>> {
        let key = self.require_session(session)?;
        let repo = self.repo(conn);
        match repo.get_by_date(date)? {
            Some(record) => {
                let note = self.decrypt_record(&record, key)?;
                Ok(Some(Self::to_response(&note)))
            }
            None => Ok(None),
        }
    }

    pub fn get_or_create_for_date(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
        date: NaiveDate,
    ) -> DomainResult<NoteResponse> {
        if let Some(existing) = self.get_by_date(conn, session, date)? {
            return Ok(existing);
        }
        let key = self.require_session(session)?;
        let now = Utc::now();
        let note = Note {
            id: Uuid::new_v4(),
            entry_date: date,
            title: String::new(),
            content: String::new(),
            tags: Vec::new(),
            is_favorite: false,
            is_done: false,
            mood: None,
            created_at: now,
            updated_at: now,
        };
        let record = self.encrypt_record(&note, key)?;
        self.repo(conn).create(&record)?;
        Ok(Self::to_response(&note))
    }

    pub fn save(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
        id: Option<String>,
        entry_date: NaiveDate,
        title: String,
        content: String,
        tags: Vec<String>,
        is_favorite: bool,
        is_done: bool,
        mood: Option<u8>,
    ) -> DomainResult<NoteResponse> {
        Self::validate_note_input(&title, &content)?;
        Self::validate_mood(mood)?;
        let key = self.require_session(session)?;
        let repo = self.repo(conn);
        let now = Utc::now();

        let mut note = if let Some(id_str) = id {
            let existing = repo.get_by_id(&id_str)?.ok_or(DomainError::NotFound)?;
            let mut n = self.decrypt_record(&existing, key)?;
            n.entry_date = entry_date;
            n.title = title;
            n.content = content;
            n.tags = tags;
            n.is_favorite = is_favorite;
            n.is_done = is_done;
            n.mood = mood;
            n.updated_at = now;
            n
        } else if let Some(existing) = repo.get_by_date(entry_date)? {
            let mut n = self.decrypt_record(&existing, key)?;
            n.title = title;
            n.content = content;
            n.tags = tags;
            n.is_favorite = is_favorite;
            n.is_done = is_done;
            n.mood = mood;
            n.updated_at = now;
            n
        } else {
            Note {
                id: Uuid::new_v4(),
                entry_date,
                title,
                content,
                tags,
                is_favorite,
                is_done,
                mood,
                created_at: now,
                updated_at: now,
            }
        };

        if Self::is_metadata_only(&note) {
            let id_str = note.id.to_string();
            if repo.get_by_id(&id_str)?.is_some() {
                repo.delete(&id_str)?;
                SyncService::record_tombstone(conn, &id_str)?;
            }
            return Ok(NoteResponse {
                id: id_str,
                entry_date: entry_date.format("%Y-%m-%d").to_string(),
                title: String::new(),
                content: String::new(),
                tags: Vec::new(),
                is_favorite: false,
                is_done: false,
                mood: None,
                created_at: note.created_at.to_rfc3339(),
                updated_at: now.to_rfc3339(),
            });
        }

        let record = self.encrypt_record(&note, key)?;
        if repo.get_by_id(&note.id.to_string())?.is_some() {
            repo.update(&record)?;
        } else {
            repo.create(&record)?;
        }
        Ok(Self::to_response(&note))
    }

    pub fn delete(&self, conn: &Connection, session: &Option<SessionKey>, id: &str) -> DomainResult<()> {
        self.require_session(session)?;
        self.repo(conn).delete(id)
    }

    pub fn toggle_favorite(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
        id: &str,
    ) -> DomainResult<NoteResponse> {
        let key = self.require_session(session)?;
        let repo = self.repo(conn);
        let record = repo.get_by_id(id)?.ok_or(DomainError::NotFound)?;
        let mut note = self.decrypt_record(&record, key)?;
        note.is_favorite = !note.is_favorite;
        note.updated_at = Utc::now();
        if Self::is_metadata_only(&note) {
            repo.delete(id)?;
            SyncService::record_tombstone(conn, id)?;
            return Ok(Self::to_response(&note));
        }
        let updated = self.encrypt_record(&note, key)?;
        repo.update(&updated)?;
        Ok(Self::to_response(&note))
    }

    pub fn list_month_markers(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
        year: i32,
        month: u32,
    ) -> DomainResult<Vec<DayMarker>> {
        let key = self.require_session(session)?;
        let records = self.repo(conn).list_for_month(year, month)?;
        let mut markers = Vec::new();
        for record in records {
            let note = self.decrypt_record(&record, key)?;
            markers.push(DayMarker {
                date: note.entry_date.format("%Y-%m-%d").to_string(),
                has_note: !note.title.is_empty() || !note.content.is_empty(),
                is_favorite: record.is_favorite,
                is_done: record.is_done,
                mood: record.mood,
            });
        }
        Ok(markers)
    }

    pub fn decrypt_all(
        &self,
        _conn: &Connection,
        session: &Option<SessionKey>,
        records: Vec<EncryptedNoteRecord>,
    ) -> DomainResult<Vec<Note>> {
        let key = self.require_session(session)?;
        records
            .iter()
            .map(|r| self.decrypt_record(r, key))
            .collect()
    }

    pub fn reencrypt_all(
        &self,
        conn: &Connection,
        old_key: &[u8; 32],
        new_key: &[u8; 32],
    ) -> DomainResult<()> {
        let repo = self.repo(conn);
        let records = repo.list_all()?;
        conn.execute("BEGIN IMMEDIATE", [])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let result = (|| -> DomainResult<()> {
            for record in records {
                let note = self.decrypt_record(&record, old_key)?;
                let reencrypted = self.encrypt_record(&note, new_key)?;
                repo.update(&reencrypted)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                conn.execute("COMMIT", [])
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(())
            }
            Err(err) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::NaiveDate;

    use crate::application::services::auth_service::SessionKey;
    use crate::application::ports::CryptoProvider;
    use crate::infrastructure::crypto::AesGcmCryptoProvider;
    use crate::infrastructure::db::connection::open_database;

    use super::*;

    #[test]
    fn reencrypt_all_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        let crypto: Arc<dyn CryptoProvider> = Arc::new(AesGcmCryptoProvider::new());
        let notes = NoteService::new(Arc::clone(&crypto));
        let salt = AesGcmCryptoProvider::generate_salt();
        let key_a = crypto.derive_key("key-a", &salt).unwrap();
        let key_b = crypto.derive_key("key-b", &salt).unwrap();
        let session_a = Some(SessionKey(key_a));
        let date = NaiveDate::from_ymd_opt(2024, 5, 30).unwrap();

        notes
            .save(
                &conn,
                &session_a,
                None,
                date,
                "Title".into(),
                "Content".into(),
                vec!["tag".into()],
                true,
                false,
                Some(3),
            )
            .unwrap();

        notes.reencrypt_all(&conn, &key_a, &key_b).unwrap();

        let session_b = Some(SessionKey(key_b));
        let note = notes.get_by_date(&conn, &session_b, date).unwrap().unwrap();
        assert_eq!(note.title, "Title");
        assert_eq!(note.content, "Content");
        assert_eq!(note.tags, vec!["tag"]);
        assert!(note.is_favorite);
        assert_eq!(note.mood, Some(3));

        assert!(notes.get_by_date(&conn, &session_a, date).is_err());
    }

    #[test]
    fn done_without_content_persists_and_clears() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        let crypto: Arc<dyn CryptoProvider> = Arc::new(AesGcmCryptoProvider::new());
        let notes = NoteService::new(Arc::clone(&crypto));
        let salt = AesGcmCryptoProvider::generate_salt();
        let key = crypto.derive_key("pw", &salt).unwrap();
        let session = Some(SessionKey(key));
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let saved = notes
            .save(
                &conn,
                &session,
                None,
                date,
                String::new(),
                String::new(),
                vec![],
                false,
                true,
                None,
            )
            .unwrap();
        assert!(saved.is_done);

        let markers = notes.list_month_markers(&conn, &session, 2024, 6).unwrap();
        assert!(markers.iter().any(|m| m.date == "2024-06-15" && m.is_done));

        notes
            .save(
                &conn,
                &session,
                Some(saved.id),
                date,
                String::new(),
                String::new(),
                vec![],
                false,
                false,
                None,
            )
            .unwrap();

        assert!(notes.get_by_date(&conn, &session, date).unwrap().is_none());
    }
}
