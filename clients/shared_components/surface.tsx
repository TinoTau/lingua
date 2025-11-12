import type { PropsWithChildren } from "react";

export type SurfaceVariant = "elevated" | "flat";

export interface SurfaceProps extends PropsWithChildren {
  variant?: SurfaceVariant;
  className?: string;
}

export function Surface({ variant = "flat", className, children }: SurfaceProps) {
  const classes = ["lingua-surface", `lingua-surface--${variant}`, className]
    .filter(Boolean)
    .join(" ");

  return <div className={classes}>{children}</div>;
}

