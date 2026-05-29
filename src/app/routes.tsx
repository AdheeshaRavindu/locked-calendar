import { Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "@/app/layout/AppShell";
import { DailyEditor } from "@/features/editor/DailyEditor";
import { CalendarView } from "@/features/calendar/CalendarView";
import { SearchView } from "@/features/search/SearchView";
import { SettingsView } from "@/features/settings/SettingsView";
import { TimelineView } from "@/features/timeline/TimelineView";

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<DailyEditor />} />
        <Route path="calendar" element={<CalendarView />} />
        <Route path="search" element={<SearchView />} />
        <Route path="timeline" element={<TimelineView />} />
        <Route
          path="favorites"
          element={
            <SearchView
              title="Favorites"
              initialFilters={{ favorites_only: true }}
            />
          }
        />
        <Route path="settings" element={<SettingsView />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
