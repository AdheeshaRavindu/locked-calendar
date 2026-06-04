import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Search, Star } from "lucide-react";
import {
  FilterBar,
  toSearchPayload,
  type FilterState,
} from "@/features/search/FilterBar";
import { EntryMeta } from "@/components/EntryMeta";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { LoadingState } from "@/components/ui/loading-state";
import { api, type NoteSummary } from "@/lib/invoke";
import { formatDisplayDate } from "@/lib/dates";
import { formatUserError } from "@/lib/errors";
import { cn } from "@/lib/utils";

const defaultFilters: FilterState = {
  query: "",
  date_from: "",
  date_to: "",
  tags: "",
  favorites_only: false,
  future_only: false,
};

function hasActiveFilters(filters: FilterState): boolean {
  return (
    filters.query.trim().length > 0 ||
    filters.date_from.length > 0 ||
    filters.date_to.length > 0 ||
    filters.tags.trim().length > 0 ||
    filters.favorites_only ||
    filters.future_only
  );
}

interface SearchViewProps {
  initialFilters?: Partial<FilterState>;
  title?: string;
}

export function SearchView({
  initialFilters,
  title = "Search",
}: SearchViewProps) {
  const navigate = useNavigate();
  const [filters, setFilters] = useState<FilterState>({
    ...defaultFilters,
    ...initialFilters,
  });
  const [results, setResults] = useState<NoteSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runSearch = useCallback(async (nextFilters: FilterState) => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.searchNotes(toSearchPayload(nextFilters));
      setResults(data);
    } catch (err) {
      setError(formatUserError(err, "Could not load search results."));
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void runSearch(filters);
    }, 300);
    return () => window.clearTimeout(timer);
  }, [filters, runSearch]);

  const isFavoritesView = title === "Favorites";
  const filtered = hasActiveFilters(filters);

  return (
    <div className="mx-auto max-w-3xl space-y-6 p-8">
      <h2 className="text-2xl font-semibold tracking-tight">{title}</h2>
      {!isFavoritesView && <FilterBar filters={filters} onChange={setFilters} />}

      {error && (
        <div className="flex items-center justify-between gap-4 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm">
          <span>{error}</span>
          <Button size="sm" variant="secondary" onClick={() => void runSearch(filters)}>
            Retry
          </Button>
        </div>
      )}

      {loading ? (
        <LoadingState label="Searching…" />
      ) : error ? null : results.length === 0 ? (
        <EmptyState
          icon={Search}
          title={
            isFavoritesView
              ? "No favorites yet"
              : filtered
                ? "No matching notes"
                : "Search your journal"
          }
          description={
            isFavoritesView
              ? "Star entries in the editor to see them here."
              : filtered
                ? "Try different keywords, dates, or tags."
                : "Search titles, content, and tags to find past entries."
          }
          actionLabel={filtered && !isFavoritesView ? "Clear filters" : undefined}
          onAction={
            filtered && !isFavoritesView
              ? () => setFilters({ ...defaultFilters, ...initialFilters })
              : undefined
          }
        />
      ) : (
        <>
          <p className="text-sm text-muted-foreground">
            {results.length} result{results.length === 1 ? "" : "s"}
            {filtered && !isFavoritesView && (
              <>
                {" · "}
                <button
                  type="button"
                  className="text-accent underline-offset-2 hover:underline"
                  onClick={() => setFilters({ ...defaultFilters, ...initialFilters })}
                >
                  Clear filters
                </button>
              </>
            )}
          </p>
          <ul className="space-y-3">
            {results.map((note) => (
              <li key={note.id}>
                <button
                  type="button"
                  onClick={() => navigate(`/?date=${note.entry_date}`)}
                  className="w-full rounded-2xl border border-border bg-card p-4 text-left shadow-card transition-colors hover:bg-muted/60"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium">{note.title}</span>
                    <span className="flex items-center gap-2">
                      <EntryMeta is_done={note.is_done} mood={note.mood} />
                    <Star
                      className={cn(
                        "h-4 w-4 shrink-0",
                        note.is_favorite
                          ? "fill-amber-400 text-amber-400"
                          : "text-transparent",
                      )}
                    />
                    </span>
                  </div>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {formatDisplayDate(note.entry_date)}
                  </p>
                  <p className="mt-2 line-clamp-2 text-sm text-muted-foreground">
                    {note.snippet || "No content"}
                  </p>
                  {note.tags.length > 0 && (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {note.tags.map((tag) => (
                        <button
                          key={tag}
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            setFilters((f) => ({
                              ...f,
                              tags: f.tags ? `${f.tags}, ${tag}` : tag,
                            }));
                          }}
                        >
                          <Badge>{tag}</Badge>
                        </button>
                      ))}
                    </div>
                  )}
                </button>
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  );
}
