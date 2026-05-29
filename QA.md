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

- [ ] `npm run tauri build` succeeds (requires Rust toolchain)
