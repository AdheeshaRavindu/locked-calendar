use crate::domain::errors::DomainResult;

pub trait CryptoProvider: Send + Sync {
    fn derive_key(&self, password: &str, salt: &[u8]) -> DomainResult<[u8; 32]>;
    fn hash_password(&self, password: &str, salt: &[u8]) -> DomainResult<Vec<u8>>;
    fn verify_password(&self, password: &str, hash: &[u8], salt: &[u8]) -> DomainResult<bool>;
    fn encrypt(&self, plaintext: &str, key: &[u8; 32]) -> DomainResult<Vec<u8>>;
    fn decrypt(&self, ciphertext: &[u8], key: &[u8; 32]) -> DomainResult<String>;
}
