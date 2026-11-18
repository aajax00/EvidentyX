import { cx } from "./cx";
import "./spinner.css";

export interface SpinnerProps {
  size?: "sm" | "md";
  /** Texte annoncé aux lecteurs d'écran. */
  label?: string;
}

export function Spinner({ size = "md", label = "Chargement" }: SpinnerProps) {
  return (
    <span role="status" className={cx("ui-spinner", `ui-spinner--${size}`)}>
      <span className="sr-only">{label}</span>
    </span>
  );
}
