# Verification checklist for Google OAuth

This file contains paste-ready text and a short demo/screenshot checklist to submit in the Google Cloud Verification Center for Locked Calendar.

---

## 1) Scope justification (paste-ready)

Short summary:

> Locked Calendar uses the Google Drive App Data scope (`https://www.googleapis.com/auth/drive.appdata`) to store a single per-user encrypted sync file in the Drive appData folder so users can securely sync their encrypted journal across devices.

Why this scope:

> `drive.appdata` gives a private app-scoped storage area that is not visible in users' main Drive UI. It is the least-privilege scope that allows a desktop app to read/write one per-user file without broad Drive access.

What is stored:

> A single JSON sync bundle named `locked-calendar-sync.json` containing AES‑GCM encrypted note fields (ciphertext only), a vault identifier, and timestamps. No plaintext user notes or Drive file listings are stored on our servers.

How access is limited & protected:

- The sync file is created only after explicit user OAuth consent in Settings → Cloud sync.
- All note content is encrypted on-device with a key derived from the user’s master password before upload; Google stores ciphertext only.
- We only read/write a single file in `appDataFolder`. Refresh tokens are encrypted locally with the session-derived key and stored locally. Users can disconnect to clear local tokens.

Reviewer checklist (what to verify):

- Connecting the app opens the Google consent screen requesting `drive.appdata` only.
- After consent, the app shows `Connected` and can perform `Sync now` which uploads/downloads the encrypted bundle.
- The Drive file contains ciphertext, not plaintext.

---

## 2) Paste-ready concise justification (one paragraph)

Locked Calendar stores one per-user encrypted sync file in the Drive `appDataFolder` to enable two-way device sync using `https://www.googleapis.com/auth/drive.appdata`. The sync bundle is encrypted on-device with AES‑GCM using a key derived from the user’s master password before upload; Google never holds plaintext. We only read/write a single app-scoped file named `locked-calendar-sync.json` after explicit user OAuth consent. Users can disconnect in Settings to remove local sync credentials.

---

## 3) Demo script (30–90s)

1. Launch the app.
2. Open Settings → Cloud sync to show current state (Not connected or Connected).
3. Click **Connect Google account**. Show the Google consent screen (scopes visible).
4. Complete consent. Return to app; show Settings now `Connected` and last sync timestamp (or empty).
5. Create or edit a note in the editor, then open Settings and click **Sync now**.
6. Show the success message: “Synced … Applied N remote update(s).”
7. (Optional) Disconnect and show Settings returns to `Not connected`.

Notes for video: blur email addresses if necessary. Show only UI flows; the reviewer can run locally if needed.

---

## 4) Screenshot checklist (file names suggested)

- `01_connect_screen.png` — Google consent screen showing `drive.appdata` scope.
- `02_settings_before.png` — Settings (Cloud sync) before connecting.
- `03_settings_after.png` — Settings showing `Connected` and last sync timestamp.
- `04_note_edit_and_sync.png` — Editor with a note and the `Sync now` button clicked.
- `05_sync_result.png` — Sync success banner: "Applied X remote update(s)".
- `06_disconnect.png` — Settings after disconnect.
- `07_app_icon_branding.png` — App About / branding page showing privacy policy URL.
- `08_encrypted_file_sample.png` (optional, reviewer-only) — screenshot of stored Drive `appDataFolder` file content showing ciphertext JSON (redact any sensitive tokens except ciphertext).

Provide a short caption for each uploaded file explaining what the reviewer should confirm.

---

## 5) "How to reproduce" text for the Verification Center (paste-ready)

Build and run the desktop app locally, then perform these steps:

```bash
npm install
npm run tauri dev
```

1. Open Locked Calendar → Settings → Cloud sync.
2. Click **Connect Google account** and complete the Google OAuth consent flow.
3. After returning to the app, create or edit a note in the editor.
4. In Settings, click **Sync now** and observe the success message.

The sync file is saved in the user’s Google Drive `appDataFolder` and contains ciphertext only. For reviewer convenience, we can supply a temporary test Google account or grant local access instructions.

---

## 6) Required assets to upload

- App homepage URL (project README or website)
- Privacy policy URL (publicly hosted) — add `privacy.html` from repo or GitHub Pages
- App icon (128×128 PNG)
- Demo video (30–90s MP4)
- Scope justification text (use content from Section 1/2 above)
- Support contact email

---

## 7) Optional: sample support text to include in verification form

If you need a short support blurb:

> Locked Calendar is an offline-first encrypted journal. Cloud sync uses Google Drive App Data to persist a single encrypted sync bundle so users can sync across devices. All note content is encrypted locally before upload; Google stores ciphertext only.

---

## 8) Add this to repo (suggested location)

- `verification-checklist.md` (this file)
- `privacy.html` (hosted URL used in verification)
- `assets/verification/*` — demo video and screenshots

---

If you want, I can add this file to the repository now (I will) and then create a `privacy.html` file from the privacy policy content we drafted earlier. Tell me to proceed if you want both files added. 

---

## Current project owner decision

- The project owner has opted to keep the OAuth client in **Testing (unverified)** mode and accept the Google-imposed limit of **up to 100 test users** for now. The files in this repo document the verification assets in case you later decide to submit for verification.

If you change your mind and want me to prepare the verification submission artifacts or walk through the verification steps, say so and I'll prepare the final package.
