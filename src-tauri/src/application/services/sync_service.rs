use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;

use crate::domain::entities::EncryptedNoteRecord;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::NoteRepository;
use crate::domain::sync::{
    SyncBundle, SyncMergeResult, SyncNoteRecord, SyncTombstone, SYNC_BUNDLE_VERSION,
};
use crate::infrastructure::db::meta_store::MetaStore;
use crate::infrastructure::repositories::SqliteNoteRepository;

pub struct SyncService;

impl SyncService {
    fn record_to_sync(record: &EncryptedNoteRecord) -> SyncNoteRecord {
        SyncNoteRecord {
            id: record.id.to_string(),
            entry_date: record.entry_date.format("%Y-%m-%d").to_string(),
            title_enc: BASE64.encode(&record.title_enc),
            content_enc: BASE64.encode(&record.content_enc),
            tags_enc: BASE64.encode(&record.tags_enc),
            is_favorite: record.is_favorite,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }

    fn sync_to_record(note: &SyncNoteRecord) -> DomainResult<EncryptedNoteRecord> {
        Ok(EncryptedNoteRecord {
            id: Uuid::parse_str(&note.id)
                .map_err(|e| DomainError::Sync(format!("Invalid note id in sync bundle: {e}")))?,
            entry_date: chrono::NaiveDate::parse_from_str(&note.entry_date, "%Y-%m-%d")
                .map_err(|e| DomainError::Sync(format!("Invalid entry date in sync bundle: {e}")))?,
            title_enc: BASE64
                .decode(&note.title_enc)
                .map_err(|e| DomainError::Sync(format!("Invalid title_enc encoding: {e}")))?,
            content_enc: BASE64
                .decode(&note.content_enc)
                .map_err(|e| DomainError::Sync(format!("Invalid content_enc encoding: {e}")))?,
            tags_enc: BASE64
                .decode(&note.tags_enc)
                .map_err(|e| DomainError::Sync(format!("Invalid tags_enc encoding: {e}")))?,
            is_favorite: note.is_favorite,
            created_at: note.created_at,
            updated_at: note.updated_at,
        })
    }

    pub fn build_bundle(conn: &Connection) -> DomainResult<SyncBundle> {
        let meta = MetaStore::new(conn);
        let vault_id = meta.ensure_vault_id()?;
        let kdf_salt = meta
            .get_kdf_salt()?
            .ok_or(DomainError::NotInitialized)?;
        let repo = SqliteNoteRepository::new(conn);
        let records = repo.list_all()?;
        let notes = records.iter().map(Self::record_to_sync).collect();
        let deleted_ids = meta.get_tombstones()?;
        Ok(SyncBundle {
            version: SYNC_BUNDLE_VERSION,
            vault_id,
            kdf_salt: BASE64.encode(&kdf_salt),
            notes,
            deleted_ids,
            synced_at: Utc::now(),
        })
    }

    pub fn record_tombstone(conn: &Connection, id: &str) -> DomainResult<()> {
        let meta = MetaStore::new(conn);
        let mut tombstones = meta.get_tombstones()?;
        let now = Utc::now();
        if let Some(existing) = tombstones.iter_mut().find(|t| t.id == id) {
            existing.deleted_at = now;
        } else {
            tombstones.push(SyncTombstone {
                id: id.to_string(),
                deleted_at: now,
            });
        }
        meta.set_tombstones(&tombstones)
    }

    fn local_note_count(conn: &Connection) -> DomainResult<usize> {
        let repo = SqliteNoteRepository::new(conn);
        Ok(repo.list_all()?.len())
    }

    pub fn adopt_remote_vault_if_empty(conn: &Connection, remote: &SyncBundle) -> DomainResult<()> {
        if Self::local_note_count(conn)? > 0 {
            return Ok(());
        }
        let meta = MetaStore::new(conn);
        let salt = BASE64
            .decode(&remote.kdf_salt)
            .map_err(|e| DomainError::Sync(format!("Invalid kdf_salt in sync bundle: {e}")))?;
        meta.set_kdf_salt(&salt)?;
        meta.set_vault_id(&remote.vault_id)?;
        Ok(())
    }

    pub fn merge_remote(conn: &Connection, remote: &SyncBundle) -> DomainResult<SyncMergeResult> {
        if remote.version != SYNC_BUNDLE_VERSION {
            return Err(DomainError::Sync(format!(
                "Unsupported sync bundle version: {}",
                remote.version
            )));
        }

        let meta = MetaStore::new(conn);
        let local_vault_id = meta.get_vault_id()?;
        let local_note_count = Self::local_note_count(conn)?;

        if local_note_count > 0 {
            if let Some(local_id) = &local_vault_id {
                if local_id != &remote.vault_id {
                    return Err(DomainError::Sync(
                        "This journal belongs to a different vault. Disconnect sync or use the same vault on all devices.".into(),
                    ));
                }
            }
        } else {
            Self::adopt_remote_vault_if_empty(conn, remote)?;
        }

        let repo = SqliteNoteRepository::new(conn);
        let local_records = repo.list_all()?;
        let mut local_by_id: std::collections::HashMap<String, EncryptedNoteRecord> =
            local_records
                .into_iter()
                .map(|r| (r.id.to_string(), r))
                .collect();

        let mut result = SyncMergeResult::default();
        let mut merged_tombstones = meta.get_tombstones()?;
        let tombstone_map: std::collections::HashMap<String, SyncTombstone> = merged_tombstones
            .iter()
            .cloned()
            .map(|t| (t.id.clone(), t))
            .collect();

        for remote_tombstone in &remote.deleted_ids {
            let local_tombstone = tombstone_map.get(&remote_tombstone.id);
            let apply_remote_tombstone = match local_tombstone {
                Some(local) => remote_tombstone.deleted_at > local.deleted_at,
                None => true,
            };
            if !apply_remote_tombstone {
                continue;
            }

            if let Some(local) = local_by_id.get(&remote_tombstone.id) {
                if remote_tombstone.deleted_at > local.updated_at {
                    repo.delete(&remote_tombstone.id)?;
                    local_by_id.remove(&remote_tombstone.id);
                    result.notes_deleted += 1;
                    result.tombstones_applied += 1;
                }
            } else if local_by_id.contains_key(&remote_tombstone.id) {
                // already handled above
            } else {
                result.tombstones_applied += 1;
            }

            if let Some(existing) = merged_tombstones.iter_mut().find(|t| t.id == remote_tombstone.id) {
                existing.deleted_at = remote_tombstone.deleted_at;
            } else {
                merged_tombstones.push(remote_tombstone.clone());
            }
        }

        for remote_note in &remote.notes {
            let remote_record = Self::sync_to_record(remote_note)?;
            let remote_id = remote_note.id.clone();

            if let Some(local_tombstone) = merged_tombstones.iter().find(|t| t.id == remote_id) {
                if local_tombstone.deleted_at >= remote_record.updated_at {
                    continue;
                }
                merged_tombstones.retain(|t| t.id != remote_id);
            }

            match local_by_id.get(&remote_id) {
                Some(local) => {
                    if remote_record.updated_at > local.updated_at {
                        repo.update(&remote_record)?;
                        result.notes_applied += 1;
                    } else {
                        result.notes_kept_local += 1;
                    }
                }
                None => {
                    repo.create(&remote_record)?;
                    result.notes_applied += 1;
                }
            }
        }

        meta.set_tombstones(&merged_tombstones)?;
        meta.ensure_vault_id()?;
        Ok(result)
    }

    pub fn merge_tombstones_local(conn: &Connection) -> DomainResult<Vec<SyncTombstone>> {
        MetaStore::new(conn).get_tombstones()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use crate::application::ports::CryptoProvider;
    use crate::application::services::auth_service::AuthService;
    use crate::application::services::note_service::NoteService;
    use crate::domain::sync::SYNC_BUNDLE_VERSION;
    use crate::infrastructure::crypto::AesGcmCryptoProvider;
    use crate::infrastructure::db::connection::open_database;
    use crate::infrastructure::db::meta_store::MetaStore;

    use super::*;

    fn setup_vault(conn: &Connection, password: &str) -> String {
        let crypto: Arc<dyn CryptoProvider> = Arc::new(AesGcmCryptoProvider::new());
        let auth = AuthService::new(Arc::clone(&crypto));
        auth.setup_master_password(conn, password).unwrap();
        MetaStore::new(conn).ensure_vault_id().unwrap()
    }

    fn save_note(conn: &Connection, password: &str, title: &str, updated_at: chrono::DateTime<Utc>) {
        let crypto: Arc<dyn CryptoProvider> = Arc::new(AesGcmCryptoProvider::new());
        let notes = NoteService::new(crypto);
        let auth = AuthService::new(Arc::new(AesGcmCryptoProvider::new()));
        let session = Some(auth.unlock(conn, password).unwrap());
        let date = chrono::NaiveDate::from_ymd_opt(2024, 5, 30).unwrap();
        let saved = notes
            .save(
                conn,
                &session,
                None,
                date,
                title.into(),
                "body".into(),
                vec![],
                false,
            )
            .unwrap();
        let repo = SqliteNoteRepository::new(conn);
        let mut record = repo.get_by_id(&saved.id).unwrap().unwrap();
        record.updated_at = updated_at;
        repo.update(&record).unwrap();
    }

    #[test]
    fn remote_newer_overwrites_local() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        let vault_id = setup_vault(&conn, "password1");
        save_note(
            &conn,
            "password1",
            "local",
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        );

        let local = SyncService::build_bundle(&conn).unwrap();
        let mut remote = local.clone();
        remote.notes[0].title_enc = BASE64.encode(b"remote ciphertext");
        remote.notes[0].updated_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let result = SyncService::merge_remote(&conn, &remote).unwrap();
        assert_eq!(result.notes_applied, 1);

        let rebuilt = SyncService::build_bundle(&conn).unwrap();
        assert_eq!(rebuilt.notes[0].title_enc, remote.notes[0].title_enc);
        let _ = vault_id;
    }

    #[test]
    fn local_newer_kept_when_remote_older() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        setup_vault(&conn, "password1");
        save_note(
            &conn,
            "password1",
            "local",
            Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        );

        let local = SyncService::build_bundle(&conn).unwrap();
        let mut remote = local.clone();
        remote.notes[0].title_enc = BASE64.encode(b"old remote");
        remote.notes[0].updated_at = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let result = SyncService::merge_remote(&conn, &remote).unwrap();
        assert_eq!(result.notes_kept_local, 1);
        assert_eq!(local.notes[0].title_enc, SyncService::build_bundle(&conn).unwrap().notes[0].title_enc);
    }

    #[test]
    fn tombstone_deletes_when_newer() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        let vault_id = setup_vault(&conn, "password1");
        save_note(
            &conn,
            "password1",
            "note",
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        );
        let note_id = SyncService::build_bundle(&conn).unwrap().notes[0].id.clone();

        let remote = SyncBundle {
            version: SYNC_BUNDLE_VERSION,
            vault_id,
            kdf_salt: MetaStore::new(&conn).get_kdf_salt().unwrap().map(|s| BASE64.encode(&s)).unwrap(),
            notes: vec![],
            deleted_ids: vec![SyncTombstone {
                id: note_id,
                deleted_at: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            }],
            synced_at: Utc::now(),
        };

        let result = SyncService::merge_remote(&conn, &remote).unwrap();
        assert_eq!(result.notes_deleted, 1);
        assert!(SqliteNoteRepository::new(&conn).list_all().unwrap().is_empty());
    }

    #[test]
    fn vault_mismatch_rejected_when_local_has_notes() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        setup_vault(&conn, "password1");
        save_note(
            &conn,
            "password1",
            "note",
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        );

        let remote = SyncBundle {
            version: SYNC_BUNDLE_VERSION,
            vault_id: Uuid::new_v4().to_string(),
            kdf_salt: BASE64.encode([1u8; 32]),
            notes: vec![],
            deleted_ids: vec![],
            synced_at: Utc::now(),
        };

        assert!(SyncService::merge_remote(&conn, &remote).is_err());
    }

    #[test]
    fn empty_local_adopts_remote_salt() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        setup_vault(&conn, "password1");

        let remote = SyncBundle {
            version: SYNC_BUNDLE_VERSION,
            vault_id: Uuid::new_v4().to_string(),
            kdf_salt: BASE64.encode([9u8; 32]),
            notes: vec![],
            deleted_ids: vec![],
            synced_at: Utc::now(),
        };

        SyncService::merge_remote(&conn, &remote).unwrap();
        let salt = MetaStore::new(&conn).get_kdf_salt().unwrap().unwrap();
        assert_eq!(salt, vec![9u8; 32]);
        assert_eq!(MetaStore::new(&conn).get_vault_id().unwrap().unwrap(), remote.vault_id);
    }
}
