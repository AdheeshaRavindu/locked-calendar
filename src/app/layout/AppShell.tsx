import { Outlet } from "react-router-dom";
import { Sidebar } from "@/app/layout/Sidebar";
import { useFocusMode } from "@/app/layout/FocusModeContext";
import { useAutoLock } from "@/hooks/useAutoLock";

export function AppShell() {
  useAutoLock(true);
  const { focusMode } = useFocusMode();

  return (
    <div className="flex h-full min-h-0 bg-background">
      {!focusMode && <Sidebar />}
      <main className="min-h-0 flex-1 overflow-auto bg-background">
        <Outlet />
      </main>
    </div>
  );
}
