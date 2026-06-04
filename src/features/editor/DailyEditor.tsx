import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import {
  Check,
  ChevronLeft,
  ChevronRight,
  Maximize2,
  Minimize2,
  Star,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { LoadingState } from "@/components/ui/loading-state";
import { Textarea } from "@/components/ui/textarea";
import { useFocusMode } from "@/app/layout/FocusModeContext";
import { ChecklistBlock } from "@/features/editor/ChecklistBlock";
import { TagInput } from "@/features/tags/TagInput";
import { OnThisDayPanel } from "@/features/timeline/OnThisDayPanel";
import { formatDisplayDate, nextDayIso, prevDayIso, todayIso } from "@/lib/dates";
import { MOOD_OPTIONS } from "@/lib/mood";
import { useNoteEditor } from "@/features/editor/useNoteEditor";
import { cn } from "@/lib/utils";

function saveLabel(status: string) {
  switch (status) {
    case "saving":
      return "Saving…";
    case "saved":
      return "Saved";
    case "error":
      return "Save failed";
    default:
      return "";
  }
}

export function DailyEditor() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const entryDate = params.get("date") ?? todayIso();
  const isToday = entryDate === todayIso();
  const { focusMode, toggleFocusMode } = useFocusMode();
  const [deleteOpen, setDeleteOpen] = useState(false);
  const {
    note,
    loading,
    saveStatus,
    updateField,
    toggleFavorite,
    deleteNote,
    hasContent,
    retrySave,
    saveNow,
  } = useNoteEditor(entryDate);

  if (loading || !note) {
    return <LoadingState label="Loading entry…" className="h-full" />;
  }

  const goToDate = (iso: string) => navigate(`/?date=${iso}`);

  const handleDelete = async () => {
    await deleteNote();
    setDeleteOpen(false);
  };

  const toggleDone = () => {
    saveNow({ ...note, is_done: !note.is_done });
  };

  const setMood = (value: number) => {
    const next = note.mood === value ? null : value;
    saveNow({ ...note, mood: next });
  };

  return (
    <div
      className={cn(
        "mx-auto flex h-full flex-col gap-8 p-8",
        focusMode ? "max-w-4xl" : "max-w-3xl",
      )}
    >
      <header className="flex items-start justify-between gap-4">
        <div className="flex items-start gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="rounded-xl"
            onClick={() => goToDate(prevDayIso(entryDate))}
            aria-label="Previous day"
          >
            <ChevronLeft className="h-5 w-5" />
          </Button>
          <div className="px-1">
            <p className="label-caps">Journal</p>
            <div className="mt-1 flex flex-wrap items-center gap-3">
              <h2 className="text-3xl font-semibold tracking-tight">
                {formatDisplayDate(note.entry_date)}
              </h2>
              {!isToday && (
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => goToDate(todayIso())}
                >
                  Back to today
                </Button>
              )}
            </div>
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="rounded-xl"
            onClick={() => goToDate(nextDayIso(entryDate))}
            aria-label="Next day"
          >
            <ChevronRight className="h-5 w-5" />
          </Button>
        </div>
        <div className="flex items-center gap-1">
          <span
            className={cn(
              "mr-2 min-w-[4.5rem] text-xs text-muted-foreground transition-opacity",
              saveStatus === "idle" ? "opacity-0" : "opacity-100",
            )}
          >
            {saveLabel(saveStatus)}
          </span>
          <Button
            variant="ghost"
            size="icon"
            className="rounded-xl"
            onClick={toggleFocusMode}
            aria-label={focusMode ? "Exit focus mode" : "Enter focus mode"}
          >
            {focusMode ? (
              <Minimize2 className="h-4 w-4" />
            ) : (
              <Maximize2 className="h-4 w-4" />
            )}
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="rounded-xl"
            onClick={() => setDeleteOpen(true)}
            disabled={!hasContent() && !note.is_done && note.mood == null}
            aria-label="Delete entry"
          >
            <Trash2 className="h-4 w-4 text-muted-foreground" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="rounded-xl"
            onClick={() => void toggleFavorite()}
            aria-label="Toggle favorite"
          >
            <Star
              className={cn(
                "h-5 w-5",
                note.is_favorite
                  ? "fill-amber-400 text-amber-400"
                  : "text-muted-foreground",
              )}
            />
          </Button>
        </div>
      </header>

      <div className="flex flex-wrap items-center gap-4">
        <button
          type="button"
          onClick={toggleDone}
          className={cn(
            "inline-flex items-center gap-2 rounded-xl border px-4 py-2 text-sm font-medium transition-colors",
            note.is_done
              ? "border-accent/30 bg-accent/12 text-accent"
              : "border-border bg-card hover:bg-muted/60",
          )}
        >
          <span
            className={cn(
              "flex h-5 w-5 items-center justify-center rounded-md border",
              note.is_done
                ? "border-accent bg-accent text-accent-foreground"
                : "border-border",
            )}
          >
            {note.is_done && <Check className="h-3 w-3" />}
          </span>
          {note.is_done ? "Day marked done" : "Mark day done"}
        </button>

        <div className="flex items-center gap-1 rounded-xl border border-border bg-card p-1">
          <span className="px-2 text-xs text-muted-foreground">Mood</span>
          {MOOD_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              type="button"
              title={opt.label}
              onClick={() => setMood(opt.value)}
              className={cn(
                "rounded-lg px-2 py-1 text-base transition-colors",
                note.mood === opt.value
                  ? "bg-accent/12 ring-1 ring-accent/40"
                  : "hover:bg-muted/60",
              )}
            >
              {opt.emoji}
            </button>
          ))}
        </div>
      </div>

      {saveStatus === "error" && (
        <div className="flex items-center justify-between gap-4 rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm">
          <span>Could not save your changes.</span>
          <Button size="sm" variant="secondary" onClick={() => void retrySave()}>
            Retry save
          </Button>
        </div>
      )}

      <OnThisDayPanel date={entryDate} compact />

      <Input
        value={note.title}
        onChange={(e) => updateField("title", e.target.value)}
        placeholder="Title (optional)"
        className="border-0 bg-transparent px-0 text-xl font-medium shadow-none focus-visible:ring-0"
      />

      <ChecklistBlock
        content={note.content}
        onChange={(content) => updateField("content", content)}
      />

      <div className="surface-inset flex min-h-[45vh] flex-1 flex-col p-1">
        <Textarea
          value={note.content}
          onChange={(e) => updateField("content", e.target.value)}
          placeholder="Write your thoughts for this day… Use - [ ] for tasks."
          className="min-h-[42vh] flex-1 border-0 bg-transparent text-base leading-relaxed shadow-none focus-visible:ring-0"
        />
      </div>

      <div className="flex items-center justify-between border-t border-border/80 pt-4 text-xs text-muted-foreground">
        <span>{note.content.length} characters</span>
      </div>

      <TagInput tags={note.tags} onChange={(tags) => updateField("tags", tags)} />

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete this entry?</DialogTitle>
            <DialogDescription>
              This permanently removes the entry for {formatDisplayDate(note.entry_date)}.
              This cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={() => void handleDelete()}>
              Delete entry
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
