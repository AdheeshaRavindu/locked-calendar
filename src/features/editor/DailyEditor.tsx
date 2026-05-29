import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import {
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
import { MarkdownPreview } from "@/features/editor/MarkdownPreview";
import { TagInput } from "@/features/tags/TagInput";
import { OnThisDayPanel } from "@/features/timeline/OnThisDayPanel";
import { formatDisplayDate, nextDayIso, prevDayIso, todayIso } from "@/lib/dates";
import { useNoteEditor } from "@/features/editor/useNoteEditor";
import { cn } from "@/lib/utils";

type EditorView = "write" | "preview" | "split";

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
  const [view, setView] = useState<EditorView>("write");
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
  } = useNoteEditor(entryDate);

  if (loading || !note) {
    return <LoadingState label="Loading entry…" className="h-full" />;
  }

  const goToDate = (iso: string) => navigate(`/?date=${iso}`);

  const handleDelete = async () => {
    await deleteNote();
    setDeleteOpen(false);
  };

  return (
    <div
      className={cn(
        "mx-auto flex h-full flex-col gap-6 p-8",
        focusMode ? "max-w-4xl" : "max-w-3xl",
      )}
    >
      <header className="flex items-start justify-between gap-4">
        <div className="flex items-start gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => goToDate(prevDayIso(entryDate))}
            aria-label="Previous day"
          >
            <ChevronLeft className="h-5 w-5" />
          </Button>
          <div>
            <p className="text-sm text-muted-foreground">Journal</p>
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="text-2xl font-semibold tracking-tight">
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
            onClick={() => goToDate(nextDayIso(entryDate))}
            aria-label="Next day"
          >
            <ChevronRight className="h-5 w-5" />
          </Button>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex rounded-lg border border-border p-0.5">
            {(["write", "preview", "split"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                onClick={() => setView(mode)}
                className={cn(
                  "rounded-md px-2.5 py-1 text-xs capitalize transition-colors",
                  view === mode
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                {mode}
              </button>
            ))}
          </div>
          <span
            className={cn(
              "text-xs text-muted-foreground transition-opacity",
              saveStatus === "idle" ? "opacity-0" : "opacity-100",
            )}
          >
            {saveLabel(saveStatus)}
          </span>
          <Button
            variant="ghost"
            size="icon"
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
            onClick={() => setDeleteOpen(true)}
            disabled={!hasContent()}
            aria-label="Delete entry"
          >
            <Trash2 className="h-4 w-4 text-muted-foreground" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
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

      {saveStatus === "error" && (
        <div className="flex items-center justify-between gap-4 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm">
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
        className="border-0 bg-transparent px-0 text-xl font-medium focus-visible:ring-0"
      />

      {view === "write" && (
        <Textarea
          value={note.content}
          onChange={(e) => updateField("content", e.target.value)}
          placeholder="Write your thoughts for this day… (Markdown supported)"
          className="min-h-[45vh] flex-1 border-0 bg-transparent px-0 text-base leading-relaxed focus-visible:ring-0"
        />
      )}

      {view === "preview" && (
        <div className="min-h-[45vh] flex-1 rounded-xl border border-border bg-card/30 p-4">
          <MarkdownPreview content={note.content} />
        </div>
      )}

      {view === "split" && (
        <div className="grid min-h-[45vh] flex-1 gap-4 lg:grid-cols-2">
          <Textarea
            value={note.content}
            onChange={(e) => updateField("content", e.target.value)}
            placeholder="Write your thoughts…"
            className="min-h-[40vh] border border-border bg-card/30 text-base leading-relaxed"
          />
          <div className="min-h-[40vh] overflow-auto rounded-xl border border-border bg-card/30 p-4">
            <MarkdownPreview content={note.content} />
          </div>
        </div>
      )}

      <div className="flex items-center justify-between border-t border-border pt-4 text-xs text-muted-foreground">
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
