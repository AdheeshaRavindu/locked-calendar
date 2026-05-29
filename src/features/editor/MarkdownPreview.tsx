import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn } from "@/lib/utils";

interface MarkdownPreviewProps {
  content: string;
  className?: string;
}

export function MarkdownPreview({ content, className }: MarkdownPreviewProps) {
  return (
    <div
      className={cn(
        "prose prose-invert max-w-none text-base leading-relaxed",
        "prose-headings:font-semibold prose-headings:text-foreground",
        "prose-p:text-foreground/90 prose-li:text-foreground/90",
        "prose-a:text-accent prose-strong:text-foreground",
        "prose-code:rounded prose-code:bg-muted prose-code:px-1 prose-code:text-foreground",
        className,
      )}
    >
      {content.trim() ? (
        <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
      ) : (
        <p className="text-muted-foreground">Nothing to preview yet.</p>
      )}
    </div>
  );
}
