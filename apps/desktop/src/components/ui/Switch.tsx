import { useId, type ReactNode } from "react";
import { cx } from "./cx";
import "./switch.css";

export interface SwitchProps {
  label: ReactNode;
  /** Explication sous le libellé. */
  description?: ReactNode;
  checked: boolean;
  disabled?: boolean;
  onChange: (checked: boolean) => void;
}

/** Interrupteur marche/arrêt (rôle ARIA « switch »), avec libellé et description. */
export function Switch({ label, description, checked, disabled, onChange }: SwitchProps) {
  const labelId = useId();
  const descriptionId = useId();
  return (
    <div className={cx("ui-switch-row", disabled && "ui-switch-row--disabled")}>
      <div className="ui-switch-row__text">
        <span id={labelId} className="ui-switch-row__label">
          {label}
        </span>
        {description && (
          <span id={descriptionId} className="ui-switch-row__description">
            {description}
          </span>
        )}
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-labelledby={labelId}
        aria-describedby={description ? descriptionId : undefined}
        disabled={disabled}
        className={cx("ui-switch", checked && "ui-switch--on")}
        onClick={() => onChange(!checked)}
      >
        <span className="ui-switch__thumb" aria-hidden="true" />
        <span className="ui-switch__state" aria-hidden="true">
          {checked ? "Oui" : "Non"}
        </span>
      </button>
    </div>
  );
}
