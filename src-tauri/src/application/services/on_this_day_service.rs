use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;

use crate::application::dto::OnThisDayEntry;
use crate::application::ports::CryptoProvider;
use crate::application::services::auth_service::SessionKey;
use crate::application::services::note_service::NoteService;
use crate::domain::entities::Note;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::NoteRepository;
use crate::infrastructure::repositories::SqliteNoteRepository;

pub struct OnThisDayService {
    note_service: NoteService,
}

impl OnThisDayService {
    pub fn new(crypto: Arc<dyn CryptoProvider>) -> Self {
        Self {
            note_service: NoteService::new(crypto),
        }
    }

    pub fn get_memories(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
        date: NaiveDate,
    ) -> DomainResult<Vec<OnThisDayEntry>> {
        let repo = SqliteNoteRepository::new(conn);
        let records = repo.list_by_month_day(date.month(), date.day())?;
        let notes = self.note_service.decrypt_all(conn, session, records)?;

        let reference_year = date.year();
        let mut entries: Vec<OnThisDayEntry> = notes
            .into_iter()
            .filter(|note| note.entry_date != date)
            .filter(|note| !note.title.is_empty() || !note.content.is_empty())
            .map(|note| to_on_this_day_entry(&note, reference_year))
            .collect();

        entries.sort_by(|a, b| b.entry_date.cmp(&a.entry_date));
        Ok(entries)
    }
}

fn to_on_this_day_entry(note: &Note, reference_year: i32) -> OnThisDayEntry {
    let snippet: String = note.content.chars().take(120).collect();
    OnThisDayEntry {
        entry_date: note.entry_date.format("%Y-%m-%d").to_string(),
        title: if note.title.is_empty() {
            "Untitled".into()
        } else {
            note.title.clone()
        },
        snippet,
        years_ago: reference_year - note.entry_date.year(),
        is_favorite: note.is_favorite,
    }
}
