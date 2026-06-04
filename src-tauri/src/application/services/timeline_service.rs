use std::collections::BTreeMap;
use std::sync::Arc;

use rusqlite::Connection;

use crate::application::dto::TimelineGroup;
use crate::application::ports::CryptoProvider;
use crate::application::services::auth_service::SessionKey;
use crate::application::services::note_service::NoteService;
use crate::domain::entities::{Note, NoteSummary};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::NoteRepository;
use crate::infrastructure::repositories::SqliteNoteRepository;

/// Groups all notes by month. Suitable for personal journals (~5k notes).
pub struct TimelineService {
    note_service: NoteService,
}

impl TimelineService {
    pub fn new(crypto: Arc<dyn CryptoProvider>) -> Self {
        Self {
            note_service: NoteService::new(crypto),
        }
    }

    pub fn list_grouped(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
    ) -> DomainResult<Vec<TimelineGroup>> {
        let repo = SqliteNoteRepository::new(conn);
        let records = repo.list_all()?;
        let notes = self.note_service.decrypt_all(conn, session, records)?;

        let mut grouped: BTreeMap<String, Vec<NoteSummary>> = BTreeMap::new();
        for note in notes {
            if note.title.is_empty()
                && note.content.is_empty()
                && !note.is_done
                && note.mood.is_none()
            {
                continue;
            }
            let month_key = note.entry_date.format("%Y-%m").to_string();
            grouped
                .entry(month_key)
                .or_default()
                .push(to_summary(&note));
        }

        let mut groups: Vec<TimelineGroup> = grouped
            .into_iter()
            .map(|(month, entries)| TimelineGroup { month, entries })
            .collect();

        groups.sort_by(|a, b| b.month.cmp(&a.month));
        for group in &mut groups {
            group.entries.sort_by(|a, b| b.entry_date.cmp(&a.entry_date));
        }
        Ok(groups)
    }
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
