use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Application is locked")]
    Locked,
    #[error("Master password has not been set up")]
    NotInitialized,
    #[error("Application is already initialized")]
    AlreadyInitialized,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Note not found")]
    NotFound,
    #[error("{0}")]
    Validation(String),
    #[error("Encryption error: {0}")]
    Crypto(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Sync error: {0}")]
    Sync(String),
}

impl DomainError {
    pub fn to_user_message(&self) -> String {
        match self {
            DomainError::Locked => "Please unlock the application first.".into(),
            DomainError::NotInitialized => "Set up a master password to continue.".into(),
            DomainError::AlreadyInitialized => "Master password is already configured.".into(),
            DomainError::InvalidPassword => "Incorrect password.".into(),
            DomainError::NotFound => "Note not found.".into(),
            DomainError::Validation(msg) => msg.clone(),
            DomainError::Crypto(_) => "Could not process encrypted data.".into(),
            DomainError::Storage(_) => "Could not access local storage.".into(),
            DomainError::Sync(msg) => msg.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<DomainError> for CommandError {
    fn from(err: DomainError) -> Self {
        CommandError {
            message: err.to_user_message(),
        }
    }
}

pub type DomainResult<T> = Result<T, DomainError>;
