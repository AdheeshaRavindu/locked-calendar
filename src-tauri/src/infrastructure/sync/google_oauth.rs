use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

use crate::domain::errors::{DomainError, DomainResult};

pub const OAUTH_REDIRECT_PORT: u16 = 8765;
pub const OAUTH_REDIRECT_PATH: &str = "/callback";
pub const DRIVE_FILE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";

#[derive(Debug, Clone)]
pub struct OAuthTokens {
    pub refresh_token: String,
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

pub fn google_client_id() -> DomainResult<String> {
    std::env::var("GOOGLE_OAUTH_CLIENT_ID").map_err(|_| {
        DomainError::Sync(
            "Google OAuth is not configured. Set GOOGLE_OAUTH_CLIENT_ID.".into(),
        )
    })
}

fn redirect_uri() -> String {
    format!("http://127.0.0.1:{OAUTH_REDIRECT_PORT}{OAUTH_REDIRECT_PATH}")
}

fn pkce_pair() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(digest);
    (verifier, challenge)
}

fn handle_callback_request(stream: &mut TcpStream) -> DomainResult<(String, String)> {
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .map_err(|e| DomainError::Sync(e.to_string()))?;
    let mut buffer = [0u8; 4096];
    let size = stream
        .read(&mut buffer)
        .map_err(|e| DomainError::Sync(format!("OAuth callback read failed: {e}")))?;
    let request = String::from_utf8_lossy(&buffer[..size]);
    let request_line = request.lines().next().unwrap_or_default();
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| DomainError::Sync("Invalid OAuth callback request.".into()))?;

    let url = Url::parse(&format!("http://127.0.0.1{path}"))
        .map_err(|e| DomainError::Sync(format!("Invalid OAuth callback URL: {e}")))?;
    let mut code = None;
    let mut state = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            _ => {}
        }
    }

    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body><h1>Locked Calendar</h1><p>You can close this window and return to the app.</p></body></html>";
    stream
        .write_all(response.as_bytes())
        .map_err(|e| DomainError::Sync(e.to_string()))?;
    stream.flush().ok();

    Ok((
        code.ok_or_else(|| DomainError::Sync("OAuth callback missing code.".into()))?,
        state.ok_or_else(|| DomainError::Sync("OAuth callback missing state.".into()))?,
    ))
}

async fn exchange_code(client_id: &str, code: &str, verifier: &str) -> DomainResult<OAuthTokens> {
    let client = Client::new();
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id),
            ("code", code),
            ("code_verifier", verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &redirect_uri()),
        ])
        .send()
        .await
        .map_err(|e| DomainError::Sync(format!("OAuth token exchange failed: {e}")))?;

    if !response.status().is_success() {
        return Err(DomainError::Sync(format!(
            "OAuth token exchange failed: {}",
            response.text().await.unwrap_or_default()
        )));
    }

    let body: TokenResponse = response
        .json()
        .await
        .map_err(|e| DomainError::Sync(format!("OAuth token parse failed: {e}")))?;
    let refresh_token = body
        .refresh_token
        .ok_or_else(|| DomainError::Sync("Google did not return a refresh token.".into()))?;

    Ok(OAuthTokens {
        refresh_token,
        access_token: body.access_token,
    })
}

pub async fn run_oauth_flow(open_url: impl FnOnce(String) -> DomainResult<()>) -> DomainResult<OAuthTokens> {
    let client_id = google_client_id()?;
    let (verifier, challenge) = pkce_pair();
    let csrf: String = (0..16)
        .map(|_| format!("{:x}", rand::random::<u8>()))
        .collect();

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri()),
        urlencoding::encode(DRIVE_FILE_SCOPE),
        urlencoding::encode(&csrf),
        urlencoding::encode(&challenge),
    );

    let listener = TcpListener::bind(format!("127.0.0.1:{OAUTH_REDIRECT_PORT}"))
        .map_err(|e| DomainError::Sync(format!("Could not start OAuth listener on port {OAUTH_REDIRECT_PORT}: {e}")))?;
    open_url(auth_url)?;

    let (code, state) = tokio::task::spawn_blocking(move || {
        let (mut stream, _) = listener
            .accept()
            .map_err(|e| DomainError::Sync(format!("OAuth callback accept failed: {e}")))?;
        handle_callback_request(&mut stream)
    })
    .await
    .map_err(|e| DomainError::Sync(format!("OAuth task failed: {e}")))??;

    if state != csrf {
        return Err(DomainError::Sync("OAuth state mismatch.".into()));
    }

    exchange_code(&client_id, &code, &verifier).await
}

pub async fn refresh_access_token(client_id: &str, refresh_token: &str) -> DomainResult<String> {
    let client = Client::new();
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| DomainError::Sync(format!("Failed to refresh Google token: {e}")))?;

    if !response.status().is_success() {
        return Err(DomainError::Sync(format!(
            "Failed to refresh Google token: {}",
            response.text().await.unwrap_or_default()
        )));
    }

    let body: TokenResponse = response
        .json()
        .await
        .map_err(|e| DomainError::Sync(format!("Refresh token parse failed: {e}")))?;
    Ok(body.access_token)
}
