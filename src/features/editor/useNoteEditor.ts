import { useCallback, useEffect, useRef, useState } from "react";
import { api, type Note } from "@/lib/invoke";

export type SaveStatus = "idle" | "saving" | "saved" | "error";

export function useNoteEditor(entryDate: string) {
  const [note, setNote] = useState<Note | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveStatus, setSaveStatus] = useState<SaveStatus>("idle");
  const debounceRef = useRef<number | null>(null);
  const noteRef = useRef<Note | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.notesGetOrCreate(entryDate);
      setNote(data);
      noteRef.current = data;
    } finally {
      setLoading(false);
    }
  }, [entryDate]);

  useEffect(() => {
    void load();
    return () => {
      if (debounceRef.current) window.clearTimeout(debounceRef.current);
    };
  }, [load]);

  const persist = useCallback(async (draft: Note) => {
    setSaveStatus("saving");
    try {
      const saved = await api.notesSave({
        id: draft.id,
        entry_date: draft.entry_date,
        title: draft.title,
        content: draft.content,
        tags: draft.tags,
        is_favorite: draft.is_favorite,
      });
      setNote(saved);
      noteRef.current = saved;
      setSaveStatus("saved");
    } catch {
      setSaveStatus("error");
    }
  }, []);

  const scheduleSave = useCallback(
    (draft: Note) => {
      noteRef.current = draft;
      setNote(draft);
      if (debounceRef.current) window.clearTimeout(debounceRef.current);
      debounceRef.current = window.setTimeout(() => {
        void persist(draft);
      }, 800);
    },
    [persist],
  );

  const updateField = useCallback(
    <K extends keyof Note>(key: K, value: Note[K]) => {
      const current = noteRef.current;
      if (!current) return;
      scheduleSave({ ...current, [key]: value });
    },
    [scheduleSave],
  );

  const toggleFavorite = useCallback(async () => {
    const current = noteRef.current;
    if (!current) return;
    const updated = await api.notesToggleFavorite(current.id);
    setNote(updated);
    noteRef.current = updated;
  }, []);

  const deleteNote = useCallback(async () => {
    const current = noteRef.current;
    if (!current) return;
    await api.notesDelete(current.id);
    await load();
    setSaveStatus("idle");
  }, [load]);

  const hasContent = useCallback(() => {
    const current = noteRef.current;
    if (!current) return false;
    return (
      current.title.trim().length > 0 ||
      current.content.trim().length > 0 ||
      current.tags.length > 0
    );
  }, []);

  return {
    note,
    loading,
    saveStatus,
    updateField,
    toggleFavorite,
    deleteNote,
    hasContent,
    reload: load,
  };
}
