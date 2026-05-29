use std::sync::Arc;

use rusqlite::Connection;
use zeroize::Zeroize;

use crate::application::ports::CryptoProvider;
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
}
