import { addDays, format, parseISO, subDays } from "date-fns";

export function formatDisplayDate(isoDate: string): string {
  return format(parseISO(isoDate), "EEEE, MMMM d, yyyy");
}

export function toIsoDate(date: Date): string {
  return format(date, "yyyy-MM-dd");
}

export function todayIso(): string {
  return toIsoDate(new Date());
}

export function shiftIsoDate(isoDate: string, deltaDays: number): string {
  const base = parseISO(isoDate);
  const shifted = deltaDays >= 0 ? addDays(base, deltaDays) : subDays(base, -deltaDays);
  return toIsoDate(shifted);
}

export function prevDayIso(isoDate: string): string {
  return toIsoDate(subDays(parseISO(isoDate), 1));
}

export function nextDayIso(isoDate: string): string {
  return toIsoDate(addDays(parseISO(isoDate), 1));
}

export function formatRelativeTime(isoDate: string): string {
  const date = parseISO(isoDate);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  if (diffSec < 60) return "just now";
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin} minute${diffMin === 1 ? "" : "s"} ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr} hour${diffHr === 1 ? "" : "s"} ago`;
  const diffDay = Math.floor(diffHr / 24);
  if (diffDay < 7) return `${diffDay} day${diffDay === 1 ? "" : "s"} ago`;
  return format(date, "MMM d, yyyy 'at' h:mm a");
}
