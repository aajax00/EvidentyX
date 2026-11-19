import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import "./empty.css";

export interface EmptyStateProps {
  icon: LucideIcon;
  title: ReactNode;
  children?: ReactNode;
  /** Action principale proposée (bouton). */
  action?: ReactNode;
}

/** État vide : explique pourquoi il n'y a rien et propose quoi faire. */
export function EmptyState({ icon: Icon, title, children, action }: EmptyStateProps) {
  return (
    <div className="ui-empty">
      <span className="ui-empty__icon" aria-hidden="true">
        <Icon size={18} />
      </span>
      <p className="ui-empty__title">{title}</p>
      {children && <div className="ui-empty__text">{children}</div>}
      {action && <div className="ui-empty__action">{action}</div>}
    </div>
  );
}
