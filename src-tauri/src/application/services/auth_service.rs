use std::sync::Arc;

use rusqlite::Connection;
use zeroize::Zeroize;

use crate::application::ports::CryptoProvider;
use crate::application::services::note_service::NoteService;
use crate::domain::errors::{DomainError, DomainResult};
use crate::infrastructure::crypto::AesGcmCryptoProvider;
use crate::infrastructure::db::meta_store::{MetaStore, KEY_PASSWORD_HASH, KEY_SALT};

#[derive(Clone)]
pub struct SessionKey(pub [u8; 32]);

impl Drop for SessionKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub struct AuthService {
    crypto: Arc<dyn CryptoProvider>,
}

impl AuthService {
    pub fn new(crypto: Arc<dyn CryptoProvider>) -> Self {
        Self { crypto }
    }

    pub fn with_default_crypto() -> Self {
        Self::new(Arc::new(AesGcmCryptoProvider::new()))
    }

    pub fn is_initialized(&self, conn: &Connection) -> DomainResult<bool> {
        MetaStore::new(conn).is_initialized()
    }

    pub fn setup_master_password(
        &self,
        conn: &Connection,
        password: &str,
    ) -> DomainResult<SessionKey> {
        if password.len() < 8 {
            return Err(DomainError::Validation(
                "Password must be at least 8 characters.".into(),
            ));
        }
        let meta = MetaStore::new(conn);
        if meta.is_initialized()? {
            return Err(DomainError::AlreadyInitialized);
        }

        let kdf_salt = AesGcmCryptoProvider::generate_salt();
        let password_hash = self.crypto.hash_password(password, &kdf_salt)?;
        let key = self.crypto.derive_key(password, &kdf_salt)?;

        meta.set(KEY_SALT, &kdf_salt)?;
        meta.set(KEY_PASSWORD_HASH, &password_hash)?;

        Ok(SessionKey(key))
    }

    pub fn unlock(&self, conn: &Connection, password: &str) -> DomainResult<SessionKey> {
        let meta = MetaStore::new(conn);
        if !meta.is_initialized()? {
            return Err(DomainError::NotInitialized);
        }

        let kdf_salt = meta
            .get(KEY_SALT)?
            .ok_or(DomainError::NotInitialized)?;
        let password_hash = meta
            .get(KEY_PASSWORD_HASH)?
            .ok_or(DomainError::NotInitialized)?;

        if !self.crypto.verify_password(password, &password_hash, &kdf_salt)? {
            return Err(DomainError::InvalidPassword);
        }

        let key = self.crypto.derive_key(password, &kdf_salt)?;
        Ok(SessionKey(key))
    }

    pub fn change_password(
        &self,
        conn: &Connection,
        note_service: &NoteService,
        old_password: &str,
        new_password: &str,
    ) -> DomainResult<SessionKey> {
        if new_password.len() < 8 {
            return Err(DomainError::Validation(
                "Password must be at least 8 characters.".into(),
            ));
        }
        if old_password == new_password {
            return Err(DomainError::Validation(
                "New password must be different from the current password.".into(),
            ));
        }

        let meta = MetaStore::new(conn);
        if !meta.is_initialized()? {
            return Err(DomainError::NotInitialized);
        }

        let old_kdf_salt = meta.get(KEY_SALT)?.ok_or(DomainError::NotInitialized)?;
        let password_hash = meta
            .get(KEY_PASSWORD_HASH)?
            .ok_or(DomainError::NotInitialized)?;

        if !self
            .crypto
            .verify_password(old_password, &password_hash, &old_kdf_salt)?
        {
            return Err(DomainError::InvalidPassword);
        }

        let old_key = self.crypto.derive_key(old_password, &old_kdf_salt)?;
        let new_kdf_salt = AesGcmCryptoProvider::generate_salt();
        let new_password_hash = self.crypto.hash_password(new_password, &new_kdf_salt)?;
        let new_key = self.crypto.derive_key(new_password, &new_kdf_salt)?;

        note_service.reencrypt_all(conn, &old_key, &new_key)?;

        meta.set(KEY_SALT, &new_kdf_salt)?;
        meta.set(KEY_PASSWORD_HASH, &new_password_hash)?;

        Ok(SessionKey(new_key))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::NaiveDate;

    use crate::application::ports::CryptoProvider;
    use crate::application::services::note_service::NoteService;
    use crate::domain::errors::DomainError;
    use crate::infrastructure::crypto::AesGcmCryptoProvider;
    use crate::infrastructure::db::connection::open_database;

    use super::*;

    #[test]
    fn change_password_rejects_wrong_current_password() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        let crypto: Arc<dyn CryptoProvider> = Arc::new(AesGcmCryptoProvider::new());
        let auth = AuthService::new(Arc::clone(&crypto));
        let notes = NoteService::new(crypto);

        let session = Some(auth.setup_master_password(&conn, "password1").unwrap());
        let date = NaiveDate::from_ymd_opt(2024, 5, 30).unwrap();
        notes
            .save(
                &conn,
                &session,
                None,
                date,
                "Title".into(),
                "Content".into(),
                vec![],
                false,
            )
            .unwrap();

        let result = auth.change_password(&conn, &notes, "wrong", "password2");
        assert!(matches!(result, Err(DomainError::InvalidPassword)));

        let new_session = Some(auth.unlock(&conn, "password1").unwrap());
        let note = notes.get_by_date(&conn, &new_session, date).unwrap().unwrap();
        assert_eq!(note.title, "Title");
        assert_eq!(note.content, "Content");
    }

    #[test]
    fn change_password_reencrypts_and_unlocks_with_new_password() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_database(&dir.path().join("test.db")).unwrap();
        let crypto: Arc<dyn CryptoProvider> = Arc::new(AesGcmCryptoProvider::new());
        let auth = AuthService::new(Arc::clone(&crypto));
        let notes = NoteService::new(crypto);

        let session = Some(auth.setup_master_password(&conn, "password1").unwrap());
        let date = NaiveDate::from_ymd_opt(2024, 5, 30).unwrap();
        notes
            .save(
                &conn,
                &session,
                None,
                date,
                "Title".into(),
                "Secret".into(),
                vec!["journal".into()],
                false,
            )
            .unwrap();

        let new_session = auth
            .change_password(&conn, &notes, "password1", "password2")
            .unwrap();
        assert!(auth.unlock(&conn, "password1").is_err());
        assert!(auth.unlock(&conn, "password2").is_ok());

        let note = notes
            .get_by_date(&conn, &Some(new_session), date)
            .unwrap()
            .unwrap();
        assert_eq!(note.content, "Secret");
        assert_eq!(note.tags, vec!["journal"]);
    }
}
