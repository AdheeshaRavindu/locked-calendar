use std::sync::Arc;

use rusqlite::Connection;

use crate::application::ports::CryptoProvider;
use crate::application::services::auth_service::SessionKey;
use crate::domain::errors::DomainResult;
use crate::infrastructure::sync::google_drive_provider::GoogleDriveProvider;

pub struct SyncProviderFactory;

impl SyncProviderFactory {
    pub fn google_drive(
        conn: &Connection,
        session: &SessionKey,
        crypto: &Arc<dyn CryptoProvider>,
    ) -> DomainResult<GoogleDriveProvider> {
        GoogleDriveProvider::from_connection(conn, session, crypto)
    }
}
