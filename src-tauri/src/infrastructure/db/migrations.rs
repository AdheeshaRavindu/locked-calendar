pub const MIGRATION_V1: &str = r#"
CREATE TABLE IF NOT EXISTS app_meta (
  key TEXT PRIMARY KEY,
  value BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  entry_date TEXT NOT NULL,
  title_enc BLOB NOT NULL,
  content_enc BLOB NOT NULL,
  tags_enc BLOB NOT NULL,
  is_favorite INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_notes_entry_date ON notes(entry_date);
CREATE INDEX IF NOT EXISTS idx_notes_favorite ON notes(is_favorite);
"#;

pub fn run_migrations(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(MIGRATION_V1)?;
    Ok(())
}
