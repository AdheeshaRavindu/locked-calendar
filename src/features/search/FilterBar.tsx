import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import type { SearchPayload } from "@/lib/invoke";

export interface FilterState {
  query: string;
  date_from: string;
  date_to: string;
  tags: string;
  favorites_only: boolean;
  future_only: boolean;
}

interface FilterBarProps {
  filters: FilterState;
  onChange: (filters: FilterState) => void;
}

export function toSearchPayload(filters: FilterState): SearchPayload {
  return {
    query: filters.query || undefined,
    date_from: filters.date_from || undefined,
    date_to: filters.date_to || undefined,
    tags: filters.tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean),
    favorites_only: filters.favorites_only,
    future_only: filters.future_only,
  };
}

export function FilterBar({ filters, onChange }: FilterBarProps) {
  const set = (partial: Partial<FilterState>) => onChange({ ...filters, ...partial });

  return (
    <div className="space-y-4 rounded-xl border border-border bg-card/50 p-4">
      <Input
        placeholder="Search titles, content, tags…"
        value={filters.query}
        onChange={(e) => set({ query: e.target.value })}
      />
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-2">
          <Label>From</Label>
          <Input
            type="date"
            value={filters.date_from}
            onChange={(e) => set({ date_from: e.target.value })}
          />
        </div>
        <div className="space-y-2">
          <Label>To</Label>
          <Input
            type="date"
            value={filters.date_to}
            onChange={(e) => set({ date_to: e.target.value })}
          />
        </div>
      </div>
      <div className="space-y-2">
        <Label>Tags (comma separated)</Label>
        <Input
          placeholder="work, health"
          value={filters.tags}
          onChange={(e) => set({ tags: e.target.value })}
        />
      </div>
      <div className="flex flex-wrap gap-6">
        <label className="flex items-center gap-2 text-sm">
          <Switch
            checked={filters.favorites_only}
            onCheckedChange={(v) => set({ favorites_only: v, future_only: v ? false : filters.future_only })}
          />
          Favorites only
        </label>
        <label className="flex items-center gap-2 text-sm">
          <Switch
            checked={filters.future_only}
            onCheckedChange={(v) => set({ future_only: v, favorites_only: v ? false : filters.favorites_only })}
          />
          Future notes only
        </label>
      </div>
    </div>
  );
}
