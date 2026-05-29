import { useNavigate, useSearchParams } from "react-router-dom";
import { ChevronLeft, ChevronRight, Star, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { TagInput } from "@/features/tags/TagInput";
import { OnThisDayPanel } from "@/features/timeline/OnThisDayPanel";
import { formatDisplayDate, nextDayIso, prevDayIso, todayIso } from "@/lib/dates";
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
  const {
    note,
    loading,
    saveStatus,
    updateField,
    toggleFavorite,
    deleteNote,
    hasContent,
  } = useNoteEditor(entryDate);

  if (loading || !note) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        Loading entry…
      </div>
    );
  }

  const goToDate = (iso: string) => navigate(`/?date=${iso}`);

  const handleDelete = () => {
    if (!hasContent()) return;
    const confirmed = window.confirm(
      "Delete this entry permanently? This cannot be undone.",
    );
    if (confirmed) void deleteNote();
  };

  return (
    <div className="mx-auto flex h-full max-w-3xl flex-col gap-6 p-8">
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
            <h2 className="text-2xl font-semibold tracking-tight">
              {formatDisplayDate(note.entry_date)}
            </h2>
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
            onClick={handleDelete}
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

      <OnThisDayPanel date={entryDate} compact />

      <Input
        value={note.title}
        onChange={(e) => updateField("title", e.target.value)}
        placeholder="Title (optional)"
        className="border-0 bg-transparent px-0 text-xl font-medium focus-visible:ring-0"
      />

      <Textarea
        value={note.content}
        onChange={(e) => updateField("content", e.target.value)}
        placeholder="Write your thoughts for this day…"
        className="min-h-[45vh] flex-1 border-0 bg-transparent px-0 text-base leading-relaxed focus-visible:ring-0"
      />

      <div className="flex items-center justify-between border-t border-border pt-4 text-xs text-muted-foreground">
        <span>{note.content.length} characters</span>
      </div>

      <TagInput
        tags={note.tags}
        onChange={(tags) => updateField("tags", tags)}
      />
    </div>
  );
}
