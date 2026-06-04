import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

const CHECKLIST_LINE = /^- \[([ xX])\] (.*)$/;

export function parseChecklistLines(content: string) {
  const lines = content.split("\n");
  const items: { lineIndex: number; checked: boolean; text: string }[] = [];
  lines.forEach((line, lineIndex) => {
    const match = line.match(CHECKLIST_LINE);
    if (match) {
      items.push({
        lineIndex,
        checked: match[1].toLowerCase() === "x",
        text: match[2],
      });
    }
  });
  return { lines, items };
}

interface ChecklistBlockProps {
  content: string;
  onChange: (content: string) => void;
}

export function ChecklistBlock({ content, onChange }: ChecklistBlockProps) {
  const { lines, items } = parseChecklistLines(content);
  if (items.length === 0) return null;

  const toggle = (lineIndex: number, checked: boolean) => {
    const next = [...lines];
    const line = next[lineIndex];
    const match = line.match(CHECKLIST_LINE);
    if (!match) return;
    next[lineIndex] = checked ? `- [x] ${match[2]}` : `- [ ] ${match[2]}`;
    onChange(next.join("\n"));
  };

  return (
    <div className="surface-inset space-y-2 p-4">
      <p className="label-caps">Tasks ({items.length})</p>
      <ul className="space-y-2">
        {items.map((item) => (
          <li key={item.lineIndex}>
            <button
              type="button"
              onClick={() => toggle(item.lineIndex, !item.checked)}
              className="flex w-full items-start gap-3 rounded-lg px-1 py-1 text-left text-sm transition-colors hover:bg-muted/50"
            >
              <span
                className={cn(
                  "mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-md border",
                  item.checked
                    ? "border-accent bg-accent text-accent-foreground"
                    : "border-border bg-card",
                )}
              >
                {item.checked && <Check className="h-3 w-3" />}
              </span>
              <span
                className={cn(
                  "flex-1",
                  item.checked && "text-muted-foreground line-through",
                )}
              >
                {item.text || "Task"}
              </span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
