use std::path::Path;

use rusqlite::Connection;

use super::migrations::run_migrations;
use crate::domain::errors::{DomainError, DomainResult};

pub fn open_database(path: &Path) -> DomainResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
    }
    let conn = Connection::open(path).map_err(|e| DomainError::Storage(e.to_string()))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| DomainError::Storage(e.to_string()))?;
    run_migrations(&conn).map_err(|e| DomainError::Storage(e.to_string()))?;
    Ok(conn)
}
