# Locked Calendar

A secure, offline-first desktop journal with a calendar interface. Notes are encrypted at rest using **AES-256-GCM** and **Argon2id** key derivation.

## Stack

- **Tauri 2** (Rust backend)
- **React + TypeScript + Vite** (frontend)
- **SQLite** (local storage, field-level encryption)

## Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

## Development

```bash
npm install
npm run tauri dev
```

### Troubleshooting `cargo` / `program not found`

Rust installs to `%USERPROFILE%\.cargo\bin`. If a terminal opened **before** Rust was installed, it may not see `cargo`.

1. **Close and reopen** PowerShell or Cursor’s terminal, then run `npm run tauri dev` again.
2. Or in the current session:
   ```powershell
   $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
   cargo --version
   npm run tauri dev
   ```
3. This project’s `npm run tauri` script prepends `.cargo\bin` automatically when needed.

### Troubleshooting `EACCES` on port 1420 (Windows)

Hyper-V and WSL can reserve TCP ports **1330–1429**, which blocks the old Tauri default **1420**. This repo uses Vite on **5173** instead. If you still see a port error, check reserved ranges: `netsh interface ipv4 show excludedportrange protocol=tcp`.

### Troubleshooting `link.exe` not found (Windows)

Install **Visual Studio Build Tools** with the **Desktop development with C++** workload, then restart the terminal:

```powershell
winget install Microsoft.VisualStudio.BuildTools --accept-package-agreements --accept-source-agreements
```

Or: https://visualstudio.microsoft.com/visual-cpp-build-tools/

## Build

```bash
npm run tauri build
```

## Architecture

```
src-tauri/src/
  domain/          # Entities, repository traits, errors
  application/     # Services, DTOs, crypto port
  infrastructure/  # SQLite, AES-GCM, Argon2, Google Drive sync
  presentation/    # Tauri commands, AppState
src/
  features/        # UI by feature (lock, editor, calendar, search)
  app/             # Layout and routes
```

## Security notes

- Master password is never stored; only a salted Argon2id hash and a separate KDF salt.
- Title, content, and tags are encrypted before being written to SQLite.
- The app auto-locks after configurable inactivity (default 10 minutes).
- **There is no password recovery** — losing your master password means losing access to encrypted notes.

## Phase 1 features

- Master password setup and lock screen
- Today-first daily editor with autosave
- Calendar view with note indicators
- Search with date range, tags, favorites, and future filters
- Favorites view
- Settings (auto-lock timeout)
- Dark premium UI

## Phase 2 features

- **Timeline** — scrollable journal history grouped by month
- **On this day** — memories from the same date in prior years
- **Tag chips** — autocomplete from existing tags
- **Editor polish** — prev/next day navigation, delete entry, character count

## Phase 2.5 features

- **Focus mode** — hide sidebar for distraction-free writing (persists in session)
- **Markdown preview** — (removed in Phase 4; use `- [ ]` task lines in the editor instead)
- **Change password** — verify current password, re-encrypt all notes, rotate KDF salt
- **Export backup** — save decrypted notes as JSON via native save dialog

## Phase 3 features

- **Google Drive sync** — two-way encrypted sync via a single Drive file (`drive.file` scope)
- **Last-write-wins merge** — per-note conflict resolution by `updated_at`
- **Manual + auto sync** — Sync now in Settings; debounced auto-sync after save/unlock (~30s)
- **Vault identity** — shared KDF salt syncs across devices (same master password required)

### Google Drive sync setup

1. Create a project in [Google Cloud Console](https://console.cloud.google.com/).
2. Enable the **Google Drive API**.
3. Configure the **OAuth consent screen** (External) and add your Google account as a test user.
4. Create an OAuth client of type **Desktop app**.
5. Add redirect URI: `http://127.0.0.1:8765/callback`
6. Set the client ID before running the app:

```powershell
$env:GOOGLE_OAUTH_CLIENT_ID = "your-client-id.apps.googleusercontent.com"
npm run tauri dev
```

The refresh token is encrypted with your session key before being stored locally.

**Note:** Cloud sync requires one-time publisher OAuth setup (`GOOGLE_OAUTH_CLIENT_ID`). Once configured, each user connects their own Google account to sync to their Drive.

## Phase 4 features

- **Mark day done** — check off a day on the calendar (with or without journal text)
- **Mood** — optional 5-level mood per day (emoji picker in the editor)
- **Checklists** — `- [ ]` / `- [x]` task lines with clickable toggles in the editor
- **Write-only editor** — Preview and Split modes removed for a simpler journaling flow
