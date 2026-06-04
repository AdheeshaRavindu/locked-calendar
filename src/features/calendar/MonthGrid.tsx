import {
  addMonths,
  eachDayOfInterval,
  endOfMonth,
  endOfWeek,
  format,
  isSameDay,
  isSameMonth,
  parseISO,
  startOfMonth,
  startOfWeek,
  subMonths,
} from "date-fns";
import { Check } from "lucide-react";
import { cn } from "@/lib/utils";
import { MOOD_OPTIONS } from "@/lib/mood";
import type { DayMarker } from "@/lib/invoke";

interface MonthGridProps {
  month: Date;
  markers: DayMarker[];
  selectedDate: string;
  onSelectDate: (iso: string) => void;
}

export function MonthGrid({
  month,
  markers,
  selectedDate,
  onSelectDate,
}: MonthGridProps) {
  const monthStart = startOfMonth(month);
  const monthEnd = endOfMonth(month);
  const days = eachDayOfInterval({
    start: startOfWeek(monthStart, { weekStartsOn: 0 }),
    end: endOfWeek(monthEnd, { weekStartsOn: 0 }),
  });

  const markerMap = new Map(markers.map((m) => [m.date, m]));
  const selected = parseISO(selectedDate);

  return (
    <div className="grid grid-cols-7 gap-1">
      {["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map((d) => (
        <div
          key={d}
          className="py-2 text-center text-xs font-medium text-muted-foreground"
        >
          {d}
        </div>
      ))}
      {days.map((day) => {
        const iso = format(day, "yyyy-MM-dd");
        const marker = markerMap.get(iso);
        const inMonth = isSameMonth(day, month);
        const isSelected = isSameDay(day, selected);
        const isToday = isSameDay(day, new Date());
        const moodEmoji = marker?.mood
          ? MOOD_OPTIONS.find((m) => m.value === marker.mood)?.emoji
          : null;

        return (
          <button
            key={iso}
            type="button"
            title={
              marker?.is_done
                ? "Done"
                : moodEmoji
                  ? `Mood: ${MOOD_OPTIONS.find((m) => m.value === marker?.mood)?.label}`
                  : undefined
            }
            onClick={() => onSelectDate(iso)}
            className={cn(
              "relative flex h-11 flex-col items-center justify-center rounded-full text-sm transition-colors",
              inMonth ? "text-foreground" : "text-muted-foreground/35",
              !isSelected && inMonth && "hover:bg-muted/70",
              isSelected && "bg-accent font-medium text-accent-foreground hover:bg-accent/90",
              isToday && !isSelected && "ring-2 ring-accent/40 ring-offset-2 ring-offset-background",
            )}
          >
            {format(day, "d")}
            <span className="absolute bottom-0 flex items-center gap-0.5">
              {marker?.is_done && (
                <Check
                  className={cn(
                    "h-2.5 w-2.5",
                    isSelected ? "text-accent-foreground" : "text-emerald-500",
                  )}
                />
              )}
              {!marker?.is_done && marker?.has_note && (
                <span
                  className={cn(
                    "h-1 w-1 rounded-full",
                    isSelected ? "bg-accent-foreground/80" : "bg-accent",
                  )}
                />
              )}
              {marker?.is_favorite && (
                <span className="h-1 w-1 rounded-full bg-amber-500" />
              )}
              {moodEmoji && (
                <span className="text-[9px] leading-none">{moodEmoji}</span>
              )}
            </span>
          </button>
        );
      })}
    </div>
  );
}

export function shiftMonth(date: Date, delta: number) {
  return delta < 0 ? subMonths(date, 1) : addMonths(date, 1);
}
