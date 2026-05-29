use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, ParamsBuilder, Version,
};
use rand::RngCore;

use crate::application::ports::CryptoProvider;
use crate::domain::errors::{DomainError, DomainResult};

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 32;

pub struct AesGcmCryptoProvider;

impl AesGcmCryptoProvider {
    pub fn new() -> Self {
        Self
    }

    fn argon2_params() -> argon2::Params {
        ParamsBuilder::new()
            .m_cost(65536)
            .t_cost(3)
            .p_cost(1)
            .build()
            .expect("valid argon2 params")
    }

    pub fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; SALT_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }
}

impl Default for AesGcmCryptoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoProvider for AesGcmCryptoProvider {
    fn derive_key(&self, password: &str, salt: &[u8]) -> DomainResult<[u8; 32]> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            Self::argon2_params(),
        );
        let mut key = [0u8; KEY_LEN];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(key)
    }

    fn hash_password(&self, password: &str, _salt: &[u8]) -> DomainResult<Vec<u8>> {
        let salt_string = SaltString::generate(&mut rand::thread_rng());
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            Self::argon2_params(),
        );
        let hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(hash.to_string().into_bytes())
    }

    fn verify_password(&self, password: &str, hash: &[u8], _salt: &[u8]) -> DomainResult<bool> {
        let hash_str = std::str::from_utf8(hash).map_err(|e| DomainError::Crypto(e.to_string()))?;
        let parsed = PasswordHash::new(hash_str).map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    }

    fn encrypt(&self, plaintext: &str, key: &[u8; 32]) -> DomainResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    fn decrypt(&self, blob: &[u8], key: &[u8; 32]) -> DomainResult<String> {
        if blob.len() < NONCE_LEN {
            return Err(DomainError::Crypto("Ciphertext too short".into()));
        }
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        String::from_utf8(plaintext).map_err(|e| DomainError::Crypto(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_roundtrip() {
        let crypto = AesGcmCryptoProvider::new();
        let salt = vec![1u8; 32];
        let key = crypto.derive_key("test-password", &salt).unwrap();
        let enc = crypto.encrypt("secret note", &key).unwrap();
        let dec = crypto.decrypt(&enc, &key).unwrap();
        assert_eq!(dec, "secret note");
    }

    #[test]
    fn argon2_deterministic_key() {
        let crypto = AesGcmCryptoProvider::new();
        let salt = vec![2u8; 32];
        let k1 = crypto.derive_key("pw", &salt).unwrap();
        let k2 = crypto.derive_key("pw", &salt).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn password_verify_fails_wrong() {
        let crypto = AesGcmCryptoProvider::new();
        let salt = AesGcmCryptoProvider::generate_salt();
        let hash = crypto.hash_password("correct", &salt).unwrap();
        assert!(!crypto.verify_password("wrong", &hash, &salt).unwrap());
        assert!(crypto.verify_password("correct", &hash, &salt).unwrap());
    }
}
