use std::sync::Arc;

use chrono::NaiveDate;
use rusqlite::Connection;

use crate::application::ports::CryptoProvider;
use crate::application::services::auth_service::SessionKey;
use crate::application::services::note_service::NoteService;
use crate::domain::entities::{Note, NoteSummary};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::NoteRepository;
use crate::infrastructure::repositories::SqliteNoteRepository;

/// In-memory search over decrypted notes. Suitable for personal journals (~5k notes).
pub struct SearchService {
    note_service: NoteService,
}

impl SearchService {
    pub fn new(crypto: Arc<dyn CryptoProvider>) -> Self {
        Self {
            note_service: NoteService::new(crypto),
        }
    }

    pub fn search(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
        query: Option<&str>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
        tags: &[String],
        favorites_only: bool,
        future_only: bool,
    ) -> DomainResult<Vec<NoteSummary>> {
        let repo = SqliteNoteRepository::new(conn);
        let today = chrono::Local::now().date_naive();

        let records = if future_only {
            repo.list_future(today)?
        } else if favorites_only {
            repo.list_favorites()?
        } else {
            repo.list_by_date_range(date_from, date_to)?
        };

        let notes = self.note_service.decrypt_all(conn, session, records)?;
        let query_lower = query.map(|q| q.to_lowercase());

        let mut results: Vec<NoteSummary> = notes
            .into_iter()
            .filter(|note| matches_tags(note, tags))
            .filter(|note| {
                if let Some(ref q) = query_lower {
                    note.title.to_lowercase().contains(q)
                        || note.content.to_lowercase().contains(q)
                        || note.tags.iter().any(|t| t.to_lowercase().contains(q))
                } else {
                    true
                }
            })
            .map(|note| to_summary(&note))
            .collect();

        results.sort_by(|a, b| b.entry_date.cmp(&a.entry_date));
        Ok(results)
    }
}

fn matches_tags(note: &Note, filter_tags: &[String]) -> bool {
    if filter_tags.is_empty() {
        return true;
    }
    filter_tags
        .iter()
        .any(|tag| note.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
}

fn to_summary(note: &Note) -> NoteSummary {
    let snippet: String = note.content.chars().take(120).collect();
    NoteSummary {
        id: note.id.to_string(),
        entry_date: note.entry_date.format("%Y-%m-%d").to_string(),
        title: if !note.title.is_empty() {
            note.title.clone()
        } else if note.is_done {
            "Day completed".into()
        } else if note.mood.is_some() {
            "Mood logged".into()
        } else {
            "Untitled".into()
        },
        snippet,
        is_favorite: note.is_favorite,
        is_done: note.is_done,
        mood: note.mood,
        tags: note.tags.clone(),
    }
}
