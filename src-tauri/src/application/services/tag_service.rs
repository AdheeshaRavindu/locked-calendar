use std::collections::BTreeSet;
use std::sync::Arc;

use rusqlite::Connection;

use crate::application::ports::CryptoProvider;
use crate::application::services::auth_service::SessionKey;
use crate::application::services::note_service::NoteService;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::NoteRepository;
use crate::infrastructure::repositories::SqliteNoteRepository;

/// Collects unique tags from all notes. Suitable for personal journals (~5k notes).
pub struct TagService {
    note_service: NoteService,
}

impl TagService {
    pub fn new(crypto: Arc<dyn CryptoProvider>) -> Self {
        Self {
            note_service: NoteService::new(crypto),
        }
    }

    pub fn list_all_tags(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
    ) -> DomainResult<Vec<String>> {
        let repo = SqliteNoteRepository::new(conn);
        let records = repo.list_all()?;
        let notes = self.note_service.decrypt_all(conn, session, records)?;

        let mut tags: BTreeSet<String> = BTreeSet::new();
        for note in notes {
            for tag in note.tags {
                let trimmed = tag.trim();
                if !trimmed.is_empty() {
                    tags.insert(trimmed.to_string());
                }
            }
        }
        Ok(tags.into_iter().collect())
    }
}
