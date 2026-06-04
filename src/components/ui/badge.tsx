import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium",
  {
    variants: {
      variant: {
        default: "border-border bg-muted text-foreground",
        accent: "border-accent/25 bg-accent/12 text-accent",
        success: "border-emerald-500/25 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400",
        muted: "border-transparent bg-muted/80 text-muted-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

export function Badge({
  className,
  variant,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & VariantProps<typeof badgeVariants>) {
  return <div className={cn(badgeVariants({ variant }), className)} {...props} />;
}
