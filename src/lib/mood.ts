export const MOOD_OPTIONS = [
  { value: 1, label: "Great", emoji: "😄" },
  { value: 2, label: "Good", emoji: "🙂" },
  { value: 3, label: "Okay", emoji: "😐" },
  { value: 4, label: "Low", emoji: "😔" },
  { value: 5, label: "Rough", emoji: "😞" },
] as const;

export function moodLabel(mood: number | null | undefined): string | null {
  if (mood == null) return null;
  return MOOD_OPTIONS.find((m) => m.value === mood)?.label ?? null;
}
