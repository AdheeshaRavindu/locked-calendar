import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { BookOpen, Star } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { LoadingState } from "@/components/ui/loading-state";
import { OnThisDayPanel } from "@/features/timeline/OnThisDayPanel";
import { api, type TimelineGroup } from "@/lib/invoke";
import { formatDisplayDate, todayIso } from "@/lib/dates";
import { formatUserError } from "@/lib/errors";
import { cn } from "@/lib/utils";

function formatMonthLabel(monthKey: string): string {
  const [year, month] = monthKey.split("-").map(Number);
  const date = new Date(year, month - 1, 1);
  return date.toLocaleDateString(undefined, { month: "long", year: "numeric" });
}

export function TimelineView() {
  const navigate = useNavigate();
  const [groups, setGroups] = useState<TimelineGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadTimeline = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.timelineList();
      setGroups(data);
    } catch (err) {
      setError(formatUserError(err, "Could not load timeline."));
      setGroups([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadTimeline();
  }, [loadTimeline]);

  const today = todayIso();

  return (
    <div className="mx-auto max-w-3xl space-y-8 p-8">
      <div>
        <h2 className="text-2xl font-semibold">Timeline</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Your journal history, grouped by month.
        </p>
      </div>

      <OnThisDayPanel date={today} />

      {error && (
        <div className="flex items-center justify-between gap-4 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm">
          <span>{error}</span>
          <Button size="sm" variant="secondary" onClick={() => void loadTimeline()}>
            Retry
          </Button>
        </div>
      )}

      {loading ? (
        <LoadingState label="Loading timeline…" />
      ) : error ? null : groups.length === 0 ? (
        <EmptyState
          icon={BookOpen}
          title="Your journal is empty"
          description="Start writing today and your entries will appear here over time."
          actionLabel="Write today's entry"
          onAction={() => navigate(`/?date=${today}`)}
        />
      ) : (
        <div className="space-y-8">
          {groups.map((group) => (
            <section key={group.month}>
              <h3 className="sticky top-0 z-10 mb-3 border-b border-border bg-background/95 py-2 text-sm font-semibold text-muted-foreground backdrop-blur">
                {formatMonthLabel(group.month)}
              </h3>
              <ul className="space-y-3">
                {group.entries.map((entry) => (
                  <li key={entry.id}>
                    <button
                      type="button"
                      onClick={() => navigate(`/?date=${entry.entry_date}`)}
                      className="w-full rounded-xl border border-border bg-card/50 p-4 text-left transition-colors hover:bg-muted/50"
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="font-medium">{entry.title}</span>
                        <Star
                          className={cn(
                            "h-4 w-4 shrink-0",
                            entry.is_favorite
                              ? "fill-amber-400 text-amber-400"
                              : "text-transparent",
                          )}
                        />
                      </div>
                      <p className="mt-1 text-xs text-muted-foreground">
                        {formatDisplayDate(entry.entry_date)}
                      </p>
                      <p className="mt-2 line-clamp-2 text-sm text-muted-foreground">
                        {entry.snippet || "No content"}
                      </p>
                      {entry.tags.length > 0 && (
                        <div className="mt-2 flex flex-wrap gap-1">
                          {entry.tags.map((tag) => (
                            <Badge key={tag}>{tag}</Badge>
                          ))}
                        </div>
                      )}
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      )}
    </div>
  );
}
