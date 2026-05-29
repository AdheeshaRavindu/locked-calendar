import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { format } from "date-fns";
import { Button } from "@/components/ui/button";
import { LoadingState } from "@/components/ui/loading-state";
import { MonthGrid, shiftMonth } from "@/features/calendar/MonthGrid";
import { api, type DayMarker } from "@/lib/invoke";
import { todayIso } from "@/lib/dates";
import { formatUserError } from "@/lib/errors";

export function CalendarView() {
  const navigate = useNavigate();
  const [month, setMonth] = useState(() => {
    const d = new Date();
    d.setDate(1);
    return d;
  });
  const [markers, setMarkers] = useState<DayMarker[]>([]);
  const [selectedDate, setSelectedDate] = useState(todayIso());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    setError(null);
    void api
      .notesListMonth(month.getFullYear(), month.getMonth() + 1)
      .then(setMarkers)
      .catch((err) => {
        setError(formatUserError(err, "Could not load calendar."));
        setMarkers([]);
      })
      .finally(() => setLoading(false));
  }, [month]);

  const openDate = (iso: string) => {
    navigate(`/?date=${iso}`);
  };

  const goToToday = () => {
    const now = new Date();
    now.setDate(1);
    setMonth(now);
    setSelectedDate(todayIso());
    openDate(todayIso());
  };

  return (
    <div className="mx-auto max-w-2xl p-8">
      <div className="mb-6 flex items-center justify-between gap-4">
        <h2 className="text-2xl font-semibold">Calendar</h2>
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={goToToday}>
            Today
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setMonth((m) => shiftMonth(m, -1))}
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <span className="min-w-[140px] text-center font-medium">
            {format(month, "MMMM yyyy")}
          </span>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setMonth((m) => shiftMonth(m, 1))}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {error && (
        <p className="mb-4 text-sm text-destructive">{error}</p>
      )}

      {loading ? (
        <LoadingState label="Loading calendar…" className="py-16" />
      ) : (
        <MonthGrid
          month={month}
          markers={markers}
          selectedDate={selectedDate}
          onSelectDate={(iso) => {
            setSelectedDate(iso);
            openDate(iso);
          }}
        />
      )}

      <p className="mt-6 text-center text-sm text-muted-foreground">
        Click a day to open or create an entry. Dots indicate notes; amber dots are favorites.
      </p>
    </div>
  );
}
