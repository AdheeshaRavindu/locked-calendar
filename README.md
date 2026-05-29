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
  infrastructure/  # SQLite, AES-GCM, Argon2
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
