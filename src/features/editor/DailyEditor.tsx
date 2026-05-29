import { useSearchParams } from "react-router-dom";
import { Star } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { formatDisplayDate, todayIso } from "@/lib/dates";
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
  const [params] = useSearchParams();
  const entryDate = params.get("date") ?? todayIso();
  const { note, loading, saveStatus, updateField, toggleFavorite } =
    useNoteEditor(entryDate);

  if (loading || !note) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        Loading entry…
      </div>
    );
  }

  const tagsInput = note.tags.join(", ");

  return (
    <div className="mx-auto flex h-full max-w-3xl flex-col gap-6 p-8">
      <header className="flex items-start justify-between gap-4">
        <div>
          <p className="text-sm text-muted-foreground">Journal</p>
          <h2 className="text-2xl font-semibold tracking-tight">
            {formatDisplayDate(note.entry_date)}
          </h2>
        </div>
        <div className="flex items-center gap-3">
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
            onClick={() => void toggleFavorite()}
            aria-label="Toggle favorite"
          >
            <Star
              className={cn(
                "h-5 w-5",
                note.is_favorite ? "fill-amber-400 text-amber-400" : "text-muted-foreground",
              )}
            />
          </Button>
        </div>
      </header>

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
        className="min-h-[50vh] flex-1 border-0 bg-transparent px-0 text-base leading-relaxed focus-visible:ring-0"
      />

      <div className="space-y-2 border-t border-border pt-4">
        <label className="text-xs font-medium text-muted-foreground">Tags (comma separated)</label>
        <Input
          value={tagsInput}
          onChange={(e) =>
            updateField(
              "tags",
              e.target.value
                .split(",")
                .map((t) => t.trim())
                .filter(Boolean),
            )
          }
          placeholder="personal, ideas, travel"
        />
        {note.tags.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {note.tags.map((tag) => (
              <Badge key={tag} variant="accent">
                {tag}
              </Badge>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
