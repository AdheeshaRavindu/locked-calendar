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
