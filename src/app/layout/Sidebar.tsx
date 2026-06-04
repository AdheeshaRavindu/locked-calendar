import { NavLink } from "react-router-dom";
import {
  Calendar,
  CalendarDays,
  Clock,
  Lock,
  Search,
  Settings,
  Star,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useSession } from "@/hooks/useSession";

type NavItem = {
  to: string;
  label: string;
  icon: LucideIcon;
  end?: boolean;
  disabled?: boolean;
};

const navItems: NavItem[] = [
  { to: "/", label: "Today", icon: CalendarDays, end: true },
  { to: "/calendar", label: "Calendar", icon: Calendar },
  { to: "/search", label: "Search", icon: Search },
  { to: "/timeline", label: "Timeline", icon: Clock },
  { to: "/favorites", label: "Favorites", icon: Star },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  const { lock } = useSession();

  return (
    <aside className="flex w-56 shrink-0 flex-col border-r border-border/50 bg-card/60 p-4 backdrop-blur-xl">
      <div className="mb-8 px-2">
        <h1 className="text-[13px] font-semibold tracking-tight">Locked Calendar</h1>
        <p className="mt-0.5 text-[11px] text-muted-foreground">Private encrypted journal</p>
      </div>
      <nav className="flex flex-1 flex-col gap-0.5">
        {navItems.map((item) =>
          item.disabled ? (
            <span
              key={item.to}
              title="Coming soon"
              className="flex cursor-not-allowed items-center gap-3 rounded-xl px-3 py-2 text-sm text-muted-foreground/50"
            >
              <item.icon className="h-4 w-4" />
              {item.label}
            </span>
          ) : (
            <NavLink
              key={item.to}
              to={item.to}
              end={"end" in item ? item.end : false}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-xl px-3 py-2 text-sm transition-colors",
                  isActive
                    ? "bg-accent/12 font-semibold text-accent"
                    : "text-muted-foreground hover:bg-muted/60 hover:text-foreground",
                )
              }
            >
              <item.icon className="h-4 w-4 shrink-0" />
              {item.label}
            </NavLink>
          ),
        )}
      </nav>
      <Button variant="ghost" className="justify-start gap-2 rounded-xl" onClick={() => void lock()}>
        <Lock className="h-4 w-4" />
        Lock now
      </Button>
    </aside>
  );
}
