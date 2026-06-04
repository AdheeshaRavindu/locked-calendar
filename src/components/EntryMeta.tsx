import { Check } from "lucide-react";
import { MOOD_OPTIONS } from "@/lib/mood";

interface EntryMetaProps {
  is_done?: boolean;
  mood?: number | null;
}

export function EntryMeta({ is_done, mood }: EntryMetaProps) {
  const emoji = mood ? MOOD_OPTIONS.find((m) => m.value === mood)?.emoji : null;
  if (!is_done && !emoji) return null;

  return (
    <span className="flex shrink-0 items-center gap-1">
      {is_done && (
        <Check className="h-3.5 w-3.5 text-emerald-500" aria-label="Done" />
      )}
      {emoji && (
        <span className="text-sm" title={MOOD_OPTIONS.find((m) => m.value === mood)?.label}>
          {emoji}
        </span>
      )}
    </span>
  );
}
