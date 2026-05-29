import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Star } from "lucide-react";
import { FilterBar, toSearchPayload, type FilterState } from "@/features/search/FilterBar";
import { Badge } from "@/components/ui/badge";
import { api, type NoteSummary } from "@/lib/invoke";
import { formatDisplayDate } from "@/lib/dates";
import { cn } from "@/lib/utils";

const defaultFilters: FilterState = {
  query: "",
  date_from: "",
  date_to: "",
  tags: "",
  favorites_only: false,
  future_only: false,
};

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

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setLoading(true);
      void api
        .searchNotes(toSearchPayload(filters))
        .then(setResults)
        .finally(() => setLoading(false));
    }, 300);
    return () => window.clearTimeout(timer);
  }, [filters]);

  return (
    <div className="mx-auto max-w-3xl space-y-6 p-8">
      <h2 className="text-2xl font-semibold">{title}</h2>
      <FilterBar filters={filters} onChange={setFilters} />
      {loading ? (
        <p className="text-sm text-muted-foreground">Searching…</p>
      ) : results.length === 0 ? (
        <p className="text-sm text-muted-foreground">No matching notes.</p>
      ) : (
        <ul className="space-y-3">
          {results.map((note) => (
            <li key={note.id}>
              <button
                type="button"
                onClick={() => navigate(`/?date=${note.entry_date}`)}
                className="w-full rounded-xl border border-border bg-card/50 p-4 text-left transition-colors hover:bg-muted/50"
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="font-medium">{note.title}</span>
                  <Star
                    className={cn(
                      "h-4 w-4 shrink-0",
                      note.is_favorite
                        ? "fill-amber-400 text-amber-400"
                        : "text-transparent",
                    )}
                  />
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
                      <Badge key={tag}>{tag}</Badge>
                    ))}
                  </div>
                )}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
