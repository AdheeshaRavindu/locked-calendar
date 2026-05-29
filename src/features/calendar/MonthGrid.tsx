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
import { cn } from "@/lib/utils";
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

        return (
          <button
            key={iso}
            type="button"
            onClick={() => onSelectDate(iso)}
            className={cn(
              "relative flex h-12 flex-col items-center justify-center rounded-lg text-sm transition-colors",
              inMonth ? "text-foreground hover:bg-muted" : "text-muted-foreground/40",
              isSelected && "bg-accent text-accent-foreground hover:bg-accent/90",
              isToday && !isSelected && "ring-1 ring-accent/50",
            )}
          >
            {format(day, "d")}
            <span className="absolute bottom-1 flex gap-0.5">
              {marker?.has_note && (
                <span
                  className={cn(
                    "h-1 w-1 rounded-full",
                    isSelected ? "bg-accent-foreground" : "bg-accent",
                  )}
                />
              )}
              {marker?.is_favorite && (
                <span className="h-1 w-1 rounded-full bg-amber-400" />
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
