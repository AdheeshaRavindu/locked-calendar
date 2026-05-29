use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, IF_MATCH};
use rusqlite::Connection;
use serde::Deserialize;

use crate::application::ports::{CryptoProvider, RemoteSyncPayload, SyncProvider};
use crate::application::services::auth_service::SessionKey;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::sync::{SyncBundle, SYNC_DRIVE_FILENAME};
use crate::infrastructure::db::meta_store::{MetaStore, KEY_SYNC_REFRESH_TOKEN_ENC};
use crate::infrastructure::sync::google_oauth::{google_client_id, refresh_access_token};

pub struct GoogleDriveProvider {
    client_id: String,
    http: reqwest::Client,
    refresh_token: String,
    file_id: Option<String>,
    etag: Option<String>,
}

impl GoogleDriveProvider {
    pub fn from_connection(conn: &Connection, session: &SessionKey, crypto: &Arc<dyn CryptoProvider>) -> DomainResult<Self> {
        let meta = MetaStore::new(conn);
        let refresh_token = Self::load_refresh_token(conn, session, crypto)?;
        Ok(Self {
            client_id: google_client_id()?,
            http: reqwest::Client::new(),
            refresh_token,
            file_id: meta.get_sync_drive_file_id()?,
            etag: meta.get_sync_drive_etag()?,
        })
    }

    pub fn store_refresh_token(
        conn: &Connection,
        session: &SessionKey,
        crypto: &Arc<dyn CryptoProvider>,
        refresh_token: &str,
    ) -> DomainResult<()> {
        let enc = crypto.encrypt(refresh_token, &session.0)?;
        MetaStore::new(conn).set(KEY_SYNC_REFRESH_TOKEN_ENC, &enc)
    }

    fn load_refresh_token(
        conn: &Connection,
        session: &SessionKey,
        crypto: &Arc<dyn CryptoProvider>,
    ) -> DomainResult<String> {
        let meta = MetaStore::new(conn);
        let enc = meta
            .get(KEY_SYNC_REFRESH_TOKEN_ENC)?
            .ok_or_else(|| DomainError::Sync("Google Drive is not connected.".into()))?;
        crypto.decrypt(&enc, &session.0)
    }

    pub async fn ensure_drive_file(
        client_id: &str,
        http: &reqwest::Client,
        refresh_token: &str,
        access_token: String,
        file_id: Option<String>,
        etag: Option<String>,
    ) -> DomainResult<(String, Option<String>)> {
        let mut provider = Self {
            client_id: client_id.to_string(),
            http: http.clone(),
            refresh_token: refresh_token.to_string(),
            file_id,
            etag,
        };
        provider.ensure_drive_file_inner(access_token).await?;
        Ok((
            provider
                .file_id
                .ok_or_else(|| DomainError::Sync("Drive file id missing.".into()))?,
            provider.etag,
        ))
    }

    async fn access_token(&self) -> DomainResult<String> {
        refresh_access_token(&self.client_id, &self.refresh_token).await
    }

    async fn ensure_drive_file_inner(&mut self, access_token: String) -> DomainResult<()> {
        if let Some(file_id) = &self.file_id {
            let meta = self.fetch_file_metadata(&access_token, file_id).await?;
            self.etag = meta.etag;
            return Ok(());
        }

        if let Some(existing) = self.find_existing_file(&access_token).await? {
            self.file_id = Some(existing.id);
            self.etag = existing.etag;
            return Ok(());
        }

        let created = self.create_empty_file(&access_token).await?;
        self.file_id = Some(created.id);
        self.etag = created.etag;
        Ok(())
    }

    async fn find_existing_file(&self, access_token: &str) -> DomainResult<Option<DriveFileMeta>> {
        let query = format!(
            "name='{}' and trashed=false",
            SYNC_DRIVE_FILENAME.replace('\'', "\\'")
        );
        let url = format!(
            "https://www.googleapis.com/drive/v3/files?q={}&spaces=drive&fields=files(id,name,etag)",
            urlencoding::encode(&query)
        );
        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive search failed: {e}")))?;
        if !response.status().is_success() {
            return Err(DomainError::Sync(format!(
                "Drive search failed: {}",
                response.text().await.unwrap_or_default()
            )));
        }
        let body: DriveFileList = response
            .json()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive search parse failed: {e}")))?;
        Ok(body.files.into_iter().next())
    }

    async fn create_empty_file(&self, access_token: &str) -> DomainResult<DriveFileMeta> {
        let metadata = serde_json::json!({
            "name": SYNC_DRIVE_FILENAME,
            "mimeType": "application/json"
        });
        let response = self
            .http
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(access_token)
            .header("Content-Type", "multipart/related; boundary=boundary_lc")
            .body(format!(
                "--boundary_lc\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n--boundary_lc\r\nContent-Type: application/json\r\n\r\n{{}}\r\n--boundary_lc--"
            ))
            .send()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive create failed: {e}")))?;
        if !response.status().is_success() {
            return Err(DomainError::Sync(format!(
                "Drive create failed: {}",
                response.text().await.unwrap_or_default()
            )));
        }
        let body: DriveFileMeta = response
            .json()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive create parse failed: {e}")))?;
        Ok(body)
    }

    async fn fetch_file_metadata(&self, access_token: &str, file_id: &str) -> DomainResult<DriveFileMeta> {
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{file_id}?fields=id,etag,name"
        );
        let response = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive metadata failed: {e}")))?;
        if !response.status().is_success() {
            return Err(DomainError::Sync(format!(
                "Drive metadata failed: {}",
                response.text().await.unwrap_or_default()
            )));
        }
        response
            .json()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive metadata parse failed: {e}")))
    }
}

#[derive(Debug, Deserialize)]
struct DriveFileList {
    files: Vec<DriveFileMeta>,
}

#[derive(Debug, Clone, Deserialize)]
struct DriveFileMeta {
    id: String,
    #[serde(default)]
    etag: Option<String>,
}

#[async_trait::async_trait]
impl SyncProvider for GoogleDriveProvider {
    async fn pull(&self) -> DomainResult<Option<RemoteSyncPayload>> {
        let file_id = self
            .file_id
            .as_ref()
            .ok_or_else(|| DomainError::Sync("Drive file is not configured.".into()))?;
        let access_token = self.access_token().await?;
        let url = format!("https://www.googleapis.com/drive/v3/files/{file_id}?alt=media");
        let response = self
            .http
            .get(url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive download failed: {e}")))?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(DomainError::Sync(format!(
                "Drive download failed: {}",
                response.text().await.unwrap_or_default()
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive download read failed: {e}")))?;
        if text.trim().is_empty() || text.trim() == "{}" {
            return Ok(None);
        }
        let bundle: SyncBundle = serde_json::from_str(&text)
            .map_err(|e| DomainError::Sync(format!("Invalid sync bundle from Drive: {e}")))?;
        Ok(Some(RemoteSyncPayload {
            bundle,
            etag: self.etag.clone(),
        }))
    }

    async fn push(&self, bundle: &SyncBundle, etag: Option<&str>) -> DomainResult<String> {
        let file_id = self
            .file_id
            .as_ref()
            .ok_or_else(|| DomainError::Sync("Drive file is not configured.".into()))?;
        let access_token = self.access_token().await?;
        let body = serde_json::to_string_pretty(bundle)
            .map_err(|e| DomainError::Sync(format!("Could not serialize sync bundle: {e}")))?;
        let url = format!(
            "https://www.googleapis.com/upload/drive/v3/files/{file_id}?uploadType=media"
        );
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        if let Some(tag) = etag {
            headers.insert(
                IF_MATCH,
                HeaderValue::from_str(tag)
                    .map_err(|e| DomainError::Sync(format!("Invalid etag: {e}")))?,
            );
        }
        let response = self
            .http
            .patch(url)
            .headers(headers)
            .bearer_auth(&access_token)
            .body(body)
            .send()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive upload failed: {e}")))?;

        if response.status().as_u16() == 412 {
            return Err(DomainError::Sync(
                "Remote sync file changed on another device. Sync again to merge.".into(),
            ));
        }
        if !response.status().is_success() {
            return Err(DomainError::Sync(format!(
                "Drive upload failed: {}",
                response.text().await.unwrap_or_default()
            )));
        }
        let meta: DriveFileMeta = response
            .json()
            .await
            .map_err(|e| DomainError::Sync(format!("Drive upload parse failed: {e}")))?;
        Ok(meta.etag.unwrap_or_else(|| "unknown".into()))
    }
}
