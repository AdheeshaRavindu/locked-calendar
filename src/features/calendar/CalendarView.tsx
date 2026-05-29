import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { format } from "date-fns";
import { Button } from "@/components/ui/button";
import { MonthGrid, shiftMonth } from "@/features/calendar/MonthGrid";
import { api, type DayMarker } from "@/lib/invoke";
import { todayIso } from "@/lib/dates";

export function CalendarView() {
  const navigate = useNavigate();
  const [month, setMonth] = useState(() => {
    const d = new Date();
    d.setDate(1);
    return d;
  });
  const [markers, setMarkers] = useState<DayMarker[]>([]);
  const [selectedDate, setSelectedDate] = useState(todayIso());

  useEffect(() => {
    void api
      .notesListMonth(month.getFullYear(), month.getMonth() + 1)
      .then(setMarkers);
  }, [month]);

  const openDate = (iso: string) => {
    navigate(`/?date=${iso}`);
  };

  return (
    <div className="mx-auto max-w-2xl p-8">
      <div className="mb-6 flex items-center justify-between">
        <h2 className="text-2xl font-semibold">Calendar</h2>
        <div className="flex items-center gap-2">
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

      <MonthGrid
        month={month}
        markers={markers}
        selectedDate={selectedDate}
        onSelectDate={(iso) => {
          setSelectedDate(iso);
          openDate(iso);
        }}
      />

      <p className="mt-6 text-center text-sm text-muted-foreground">
        Click a day to open or create an entry. Dots indicate notes; amber dots are favorites.
      </p>
    </div>
  );
}
