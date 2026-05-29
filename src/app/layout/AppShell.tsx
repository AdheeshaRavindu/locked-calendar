import { Outlet } from "react-router-dom";
import { Sidebar } from "@/app/layout/Sidebar";
import { useAutoLock } from "@/hooks/useAutoLock";

export function AppShell() {
  useAutoLock(true);

  return (
    <div className="flex h-full min-h-0">
      <Sidebar />
      <main className="min-h-0 flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
