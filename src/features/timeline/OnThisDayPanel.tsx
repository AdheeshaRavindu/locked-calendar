import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Star } from "lucide-react";
import { LoadingState } from "@/components/ui/loading-state";
import { api, type OnThisDayEntry } from "@/lib/invoke";
import { cn } from "@/lib/utils";

interface OnThisDayPanelProps {
  date: string;
  compact?: boolean;
}

export function OnThisDayPanel({ date, compact = false }: OnThisDayPanelProps) {
  const navigate = useNavigate();
  const [entries, setEntries] = useState<OnThisDayEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    void api
      .notesOnThisDay(date)
      .then(setEntries)
      .finally(() => setLoading(false));
  }, [date]);

  if (loading) {
    return <LoadingState label="Loading memories…" className="py-4" />;
  }

  if (entries.length === 0) {
    return null;
  }

  return (
    <section
      className={cn(
        "rounded-xl border border-border bg-card/50",
        compact ? "p-4" : "p-5",
      )}
    >
      <h3 className="mb-3 text-sm font-medium text-muted-foreground">
        On this day
      </h3>
      <ul className="space-y-2">
        {entries.map((entry) => (
          <li key={entry.entry_date}>
            <button
              type="button"
              onClick={() => navigate(`/?date=${entry.entry_date}`)}
              className="w-full rounded-lg border border-border/60 bg-background/50 p-3 text-left transition-colors hover:bg-muted/50"
            >
              <div className="flex items-center justify-between gap-2">
                <span className="text-sm font-medium">{entry.title}</span>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground">
                    {entry.years_ago === 1
                      ? "1 year ago"
                      : `${entry.years_ago} years ago`}
                  </span>
                  <Star
                    className={cn(
                      "h-3.5 w-3.5",
                      entry.is_favorite
                        ? "fill-amber-400 text-amber-400"
                        : "text-transparent",
                    )}
                  />
                </div>
              </div>
              {entry.snippet && (
                <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">
                  {entry.snippet}
                </p>
              )}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
