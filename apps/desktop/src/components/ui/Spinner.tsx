import { cx } from "./cx";
import "./spinner.css";

export interface SpinnerProps {
  size?: "sm" | "md";
  /** Texte annoncé aux lecteurs d'écran. */
  label?: string;
}

/** Chargeur « X » : anneau tournant autour du X bleu de la marque (seul élément circulaire de l'interface). */
export function Spinner({ size = "md", label = "Chargement" }: SpinnerProps) {
  return (
    <span role="status" className={cx("ui-spinner", `ui-spinner--${size}`)}>
      <span className="ui-spinner__ring" aria-hidden="true" />
      <span className="ui-spinner__x" aria-hidden="true">
        X
      </span>
      <span className="sr-only">{label}</span>
    </span>
  );
}
