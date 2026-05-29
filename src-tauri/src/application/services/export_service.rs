use std::sync::Arc;

use chrono::Utc;
use rusqlite::Connection;
use serde::Serialize;

use crate::application::dto::NoteResponse;
use crate::application::ports::CryptoProvider;
use crate::application::services::auth_service::SessionKey;
use crate::application::services::note_service::NoteService;
use crate::domain::errors::DomainResult;
use crate::domain::repositories::NoteRepository;
use crate::infrastructure::repositories::SqliteNoteRepository;

#[derive(Debug, Serialize)]
pub struct ExportPayload {
    pub version: u32,
    pub exported_at: String,
    pub notes: Vec<NoteResponse>,
}

pub struct ExportService {
    note_service: NoteService,
}

impl ExportService {
    pub fn new(crypto: Arc<dyn CryptoProvider>) -> Self {
        Self {
            note_service: NoteService::new(crypto),
        }
    }

    pub fn export_json(
        &self,
        conn: &Connection,
        session: &Option<SessionKey>,
    ) -> DomainResult<String> {
        let repo = SqliteNoteRepository::new(conn);
        let records = repo.list_all()?;
        let notes = self.note_service.decrypt_all(conn, session, records)?;
        let responses: Vec<NoteResponse> = notes.iter().map(NoteService::to_response).collect();
        let payload = ExportPayload {
            version: 1,
            exported_at: Utc::now().to_rfc3339(),
            notes: responses,
        };
        serde_json::to_string_pretty(&payload)
            .map_err(|e| crate::domain::errors::DomainError::Storage(e.to_string()))
    }
}
