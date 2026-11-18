import { useId, type ReactNode } from "react";
import { cx } from "./cx";
import "./panel.css";

export interface PanelProps {
  title?: ReactNode;
  /** Éléments placés à droite du titre (badge, actions). */
  actions?: ReactNode;
  className?: string;
  children: ReactNode;
}

export function Panel({ title, actions, className, children }: PanelProps) {
  const titleId = useId();
  return (
    <section className={cx("ui-panel", className)} aria-labelledby={title ? titleId : undefined}>
      {(title || actions) && (
        <header className="ui-panel__header">
          {title && (
            <h2 id={titleId} className="ui-panel__title">
              {title}
            </h2>
          )}
          {actions && <div className="ui-panel__actions">{actions}</div>}
        </header>
      )}
      <div className="ui-panel__body">{children}</div>
    </section>
  );
}
