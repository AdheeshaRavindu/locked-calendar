use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SetupPasswordRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UnlockRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveNoteRequest {
    pub id: Option<String>,
    pub entry_date: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub is_favorite: bool,
}

#[derive(Debug, Deserialize)]
pub struct SearchNotesRequest {
    pub query: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub tags: Vec<String>,
    pub favorites_only: bool,
    pub future_only: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub initialized: bool,
    pub unlocked: bool,
}

#[derive(Debug, Serialize)]
pub struct TimelineGroup {
    pub month: String,
    pub entries: Vec<crate::domain::entities::NoteSummary>,
}

#[derive(Debug, Serialize)]
pub struct OnThisDayEntry {
    pub entry_date: String,
    pub title: String,
    pub snippet: String,
    pub years_ago: i32,
    pub is_favorite: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteResponse {
    pub id: String,
    pub entry_date: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}
