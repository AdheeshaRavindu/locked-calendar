# Locked Calendar — Phase 1 QA Checklist

## Authentication

- [ ] First run: setup password → lands on today
- [ ] Wrong password rejected on unlock
- [ ] Manual lock from sidebar and settings
- [ ] Auto-lock after idle period

## Notes

- [ ] Create note today, restart app, unlock, note persists
- [ ] Create note on past/future date via calendar
- [ ] Autosave indicator appears while editing
- [ ] Favorite toggle updates star state

## Calendar

- [ ] Calendar shows dot indicators on days with notes
- [ ] Clicking a day opens editor for that date

## Search & filters

- [ ] Search finds text in title and body
- [ ] Date range filter works
- [ ] Tag filter works
- [ ] Favorites-only filter works
- [ ] Future-only filter works
- [ ] Favorites sidebar view lists starred notes

## Security

- [ ] Inspect SQLite DB: note title/content/tags are not readable plaintext

## Build

- [x] `npm run tauri build` succeeds (requires Rust toolchain)

## Phase 2

- [x] Timeline lists all notes grouped by month, newest first
- [x] Clicking timeline entry opens correct date in editor
- [x] On this day shows entries from prior years on same month/day
- [x] Tag chips add/remove correctly; suggestions appear from existing tags
- [x] Clicking tag in search results adds tag filter
- [x] Prev/next day navigation works across month boundaries
- [x] Delete note removes entry and updates calendar dots

## Phase 2.5

- [ ] Focus mode hides sidebar; toggle persists while navigating
- [ ] Markdown preview removed (Phase 4); checklist `- [ ]` toggles work in editor
- [x] Change password re-encrypts notes; unlock with new password works (Rust tests)
- [x] Change password with wrong current password rejected; notes intact (Rust tests)
- [ ] Export backup produces valid JSON via save dialog
- [ ] Restart app after password change; new password works

## Phase 3

_Deferred — requires publisher `GOOGLE_OAUTH_CLIENT_ID` setup. End users connect their own Google account once configured._

- [ ] `GOOGLE_OAUTH_CLIENT_ID` set; Connect Google Drive opens browser and completes OAuth
- [ ] Initial sync creates `locked-calendar-sync.json` on Drive
- [ ] Second device: connect + sync → notes appear with same master password
- [ ] Edit on device A, sync; edit on device B, sync → last-write-wins result correct
- [ ] Delete on one device propagates after sync
- [ ] Disconnect clears tokens; reconnect works
- [ ] Auto-sync fires ~30s after note save
- [ ] Drive file contains encrypted blobs only (no plaintext title/content)

## Phase 4

- [ ] Mark day done without writing text; calendar shows checkmark; clearing done removes empty row
- [ ] Mood picker sets and clears mood; calendar shows emoji; persists after restart
- [ ] Checklist lines `- [ ]` appear in Tasks section; clicking toggles to `- [x]` and saves
- [ ] Export JSON includes `is_done` and `mood` fields
- [ ] Sync merge preserves `is_done` and `mood` across devices (when Phase 3 OAuth configured)
