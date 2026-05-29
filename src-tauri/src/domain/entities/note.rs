use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub entry_date: NaiveDate,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EncryptedNoteRecord {
    pub id: Uuid,
    pub entry_date: NaiveDate,
    pub title_enc: Vec<u8>,
    pub content_enc: Vec<u8>,
    pub tags_enc: Vec<u8>,
    pub is_favorite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoteSummary {
    pub id: String,
    pub entry_date: String,
    pub title: String,
    pub snippet: String,
    pub is_favorite: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayMarker {
    pub date: String,
    pub has_note: bool,
    pub is_favorite: bool,
}
