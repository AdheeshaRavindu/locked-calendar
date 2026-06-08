# Demo video captions & screenshot captions

Use these captions when uploading the demo video and screenshots to Google Verification Center.

## Video captions (use as short chapter headings)

1. Launch app — show main window and Settings menu.
2. Open Settings → Cloud sync (show current state: Not connected).
3. Connect Google account — display Google consent screen (scopes shown).
4. Return to app — Settings shows Connected and last sync timestamp.
5. Create or edit a note in the editor.
6. Perform Sync now — show success message (Applied N remote update(s)).
7. Disconnect — show Settings returns to Not connected.

## Screenshot captions (pair with file names)

- `01_connect_screen.png` — Google consent screen listing requested scope: "View and manage its own configuration data in your Google Drive" (appData scope).
- `02_settings_before.png` — Settings → Cloud sync showing Not connected.
- `03_settings_after.png` — Settings → Cloud sync showing Connected and last sync timestamp.
- `04_note_edit_and_sync.png` — Editor with note text and Sync now clicked.
- `05_sync_result.png` — Sync success banner: "Synced ... Applied N remote update(s)."
- `06_disconnect.png` — Settings after Disconnect showing Not connected.
- `07_app_icon_branding.png` — App About or Settings page showing app icon and privacy policy URL.
- `08_encrypted_file_sample.png` — (optional) screenshot of the file contents in Drive `appDataFolder` demonstrating ciphertext (redact tokens).

## Additional notes for reviewers

- The sync bundle is encrypted client-side. The Drive file is a JSON blob of base64 ciphertext fields; reviewers may inspect that blob to confirm no plaintext is present.
- If the reviewer needs access, provide a temporary test Google account and a time window, or ask the reviewer to run the app locally and follow the demo script above.
