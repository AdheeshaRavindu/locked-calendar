import { format, parseISO } from "date-fns";

export function formatDisplayDate(isoDate: string): string {
  return format(parseISO(isoDate), "EEEE, MMMM d, yyyy");
}

export function toIsoDate(date: Date): string {
  return format(date, "yyyy-MM-dd");
}

export function todayIso(): string {
  return toIsoDate(new Date());
}
