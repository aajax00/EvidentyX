import { useEffect, useId, useRef, type ReactNode } from "react";
import { X } from "lucide-react";
import { IconButton } from "./Button";
import "./dialog.css";

export interface DialogProps {
  title: ReactNode;
  /** Texte d'introduction sous le titre. */
  description?: ReactNode;
  onClose: () => void;
  /** Boutons d'action, alignés à droite. */
  footer?: ReactNode;
  width?: "sm" | "md";
  children?: ReactNode;
}

/**
 * Boîte de dialogue modale, basée sur l'élément natif `<dialog>` :
 * piège à focus, fermeture par Échap et retour du focus fournis par le navigateur.
 * Le composant est monté quand la boîte est ouverte, démonté à la fermeture.
 */
export function Dialog({ title, description, onClose, footer, width = "md", children }: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const descriptionId = useId();
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialog.showModal();
    const onCancel = (event: Event) => {
      event.preventDefault();
      onCloseRef.current();
    };
    dialog.addEventListener("cancel", onCancel);
    return () => {
      dialog.removeEventListener("cancel", onCancel);
      dialog.close();
      previous?.focus();
    };
  }, []);

  return (
    <dialog
      ref={ref}
      className={`ui-dialog ui-dialog--${width}`}
      aria-labelledby={titleId}
      aria-describedby={description ? descriptionId : undefined}
      onMouseDown={(e) => {
        // Clic sur le fond (hors du contenu) : fermeture.
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="ui-dialog__surface">
        <header className="ui-dialog__header">
          <h2 id={titleId} className="ui-dialog__title">
            {title}
          </h2>
          <IconButton icon={X} label="Fermer" onClick={onClose} />
        </header>
        {description && (
          <p id={descriptionId} className="ui-dialog__description">
            {description}
          </p>
        )}
        {children && <div className="ui-dialog__body">{children}</div>}
        {footer && <footer className="ui-dialog__footer">{footer}</footer>}
      </div>
    </dialog>
  );
}
