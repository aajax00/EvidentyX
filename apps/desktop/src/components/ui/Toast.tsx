import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from "react";
import { CircleAlert, CircleCheck, X } from "lucide-react";
import { cx } from "./cx";
import "./toast.css";

export type ToastTone = "success" | "danger";

interface ToastItem {
  id: number;
  tone: ToastTone;
  message: string;
}

interface ToastApi {
  /** Affiche une notification brève (fermée automatiquement après 5 s). */
  notify: (message: string, tone?: ToastTone) => void;
}

const ToastContext = createContext<ToastApi | null>(null);

const DURATION_MS = 5000;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);
  const nextId = useRef(1);

  const dismiss = useCallback((id: number) => setItems((list) => list.filter((t) => t.id !== id)), []);

  const notify = useCallback(
    (message: string, tone: ToastTone = "success") => {
      const id = nextId.current++;
      setItems((list) => [...list.slice(-3), { id, tone, message }]);
      window.setTimeout(() => dismiss(id), DURATION_MS);
    },
    [dismiss],
  );

  const api = useMemo(() => ({ notify }), [notify]);

  return (
    <ToastContext.Provider value={api}>
      {children}
      <div className="ui-toasts" role="status" aria-live="polite">
        {items.map((t) => {
          const Icon = t.tone === "success" ? CircleCheck : CircleAlert;
          return (
            <div key={t.id} className={cx("ui-toast", `ui-toast--${t.tone}`)}>
              <Icon size={15} aria-hidden="true" className="ui-toast__icon" />
              <span className="ui-toast__message">{t.message}</span>
              <button type="button" className="ui-toast__close" aria-label="Fermer la notification" onClick={() => dismiss(t.id)}>
                <X size={13} aria-hidden="true" />
              </button>
              <span className="ui-toast__progress" aria-hidden="true" />
            </div>
          );
        })}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast(): ToastApi {
  const api = useContext(ToastContext);
  if (!api) throw new Error("useToast doit être utilisé sous <ToastProvider>");
  return api;
}
