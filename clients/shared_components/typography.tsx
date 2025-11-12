import type { PropsWithChildren } from "react";

export type TypographyVariant = "title" | "subtitle" | "body" | "caption";

export interface TypographyProps extends PropsWithChildren {
  variant?: TypographyVariant;
  className?: string;
}

export function Typography({ variant = "body", className, children }: TypographyProps) {
  const classes = ["lingua-typography", `lingua-typography--${variant}`, className]
    .filter(Boolean)
    .join(" ");

  switch (variant) {
    case "title":
      return <h1 className={classes}>{children}</h1>;
    case "subtitle":
      return <h2 className={classes}>{children}</h2>;
    case "caption":
      return <span className={classes}>{children}</span>;
    default:
      return <p className={classes}>{children}</p>;
  }
}

