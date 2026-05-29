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
};
