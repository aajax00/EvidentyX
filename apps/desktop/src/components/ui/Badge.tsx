import type { ReactNode } from "react";
import { cx } from "./cx";
import "./badge.css";

export type BadgeTone = "neutral" | "accent" | "success" | "warning" | "danger" | "info";

export interface BadgeProps {
  tone?: BadgeTone;
  children: ReactNode;
}

export function Badge({ tone = "neutral", children }: BadgeProps) {
  return <span className={cx("ui-badge", `ui-badge--${tone}`)}>{children}</span>;
}
