import { invoke } from "@tauri-apps/api/core";

export interface AuthStatus {
  initialized: boolean;
  unlocked: boolean;
}

export interface Note {
  id: string;
  entry_date: string;
  title: string;
  content: string;
  tags: string[];
  is_favorite: boolean;
  created_at: string;
  updated_at: string;
}

export interface NoteSummary {
  id: string;
  entry_date: string;
  title: string;
  snippet: string;
  is_favorite: boolean;
  tags: string[];
}

export interface DayMarker {
  date: string;
  has_note: boolean;
  is_favorite: boolean;
}

export interface SaveNotePayload {
  id?: string;
  entry_date: string;
  title: string;
  content: string;
  tags: string[];
  is_favorite: boolean;
}

export interface SearchPayload {
  query?: string;
  date_from?: string;
  date_to?: string;
  tags: string[];
  favorites_only: boolean;
  future_only: boolean;
}

export interface TimelineGroup {
  month: string;
  entries: NoteSummary[];
}

export interface OnThisDayEntry {
  entry_date: string;
  title: string;
  snippet: string;
  years_ago: number;
  is_favorite: boolean;
}

export interface SyncStatus {
  connected: boolean;
  last_sync_at: string | null;
  in_progress: boolean;
  error: string | null;
}

export interface SyncMergeResult {
  notes_applied: number;
  notes_kept_local: number;
  tombstones_applied: number;
  notes_deleted: number;
}

export interface SyncNowResult {
  merged: SyncMergeResult;
  pushed: boolean;
  last_sync_at: string;
}

export const api = {
  authIsInitialized: () => invoke<boolean>("auth_is_initialized"),
  authStatus: () => invoke<AuthStatus>("auth_status"),
  authSetup: (password: string) => invoke<void>("auth_setup", { password }),
  authUnlock: (password: string) => invoke<void>("auth_unlock", { password }),
  authLock: () => invoke<void>("auth_lock"),
  authTouchSession: () => invoke<void>("auth_touch_session"),
  authGetLockTimeout: () => invoke<number>("auth_get_lock_timeout"),
  authSetLockTimeout: (seconds: number) =>
    invoke<void>("auth_set_lock_timeout", { seconds }),
  authChangePassword: (oldPassword: string, newPassword: string) =>
    invoke<void>("auth_change_password", {
      oldPassword,
      newPassword,
    }),
  exportNotesJson: () => invoke<string>("export_notes_json"),
  notesGetToday: () => invoke<Note>("notes_get_today"),
  notesGetByDate: (date: string) =>
    invoke<Note | null>("notes_get_by_date", { date }),
  notesGetOrCreate: (date: string) =>
    invoke<Note>("notes_get_or_create", { date }),
  notesSave: (payload: SaveNotePayload) =>
    invoke<Note>("notes_save", { payload }),
  notesDelete: (id: string) => invoke<void>("notes_delete", { id }),
  notesToggleFavorite: (id: string) =>
    invoke<Note>("notes_toggle_favorite", { id }),
  notesListMonth: (year: number, month: number) =>
    invoke<DayMarker[]>("notes_list_month", { year, month }),
  searchNotes: (payload: SearchPayload) =>
    invoke<NoteSummary[]>("search_notes", { payload }),
  timelineList: () => invoke<TimelineGroup[]>("timeline_list"),
  notesOnThisDay: (date: string) =>
    invoke<OnThisDayEntry[]>("notes_on_this_day", { date }),
  tagsList: () => invoke<string[]>("tags_list"),
  syncConnect: () => invoke<{ connected: boolean }>("sync_connect"),
  syncDisconnect: () => invoke<void>("sync_disconnect"),
  syncNow: () => invoke<SyncNowResult>("sync_now"),
  syncStatus: () => invoke<SyncStatus>("sync_status"),
};
