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

fn column_exists(conn: &rusqlite::Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let sql = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn migrate_v2(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    if !column_exists(conn, "notes", "is_done")? {
        conn.execute(
            "ALTER TABLE notes ADD COLUMN is_done INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !column_exists(conn, "notes", "mood")? {
        conn.execute("ALTER TABLE notes ADD COLUMN mood INTEGER", [])?;
    }
    Ok(())
}

pub fn run_migrations(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(MIGRATION_V1)?;
    migrate_v2(conn)?;
    Ok(())
}
