import { useId, type KeyboardEvent, type ReactNode } from "react";
import { cx } from "./cx";
import "./segmented.css";

export interface SegmentOption<T extends string> {
  value: T;
  label: ReactNode;
  /** Compteur affiché après le libellé. */
  count?: number;
}

export interface SegmentedControlProps<T extends string> {
  label: string;
  value: T;
  options: readonly SegmentOption<T>[];
  onChange: (value: T) => void;
}

/** Choix exclusif parmi quelques options (groupe radio accessible, flèches gauche/droite). */
export function SegmentedControl<T extends string>({ label, value, options, onChange }: SegmentedControlProps<T>) {
  const name = useId();

  function onKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    const index = options.findIndex((o) => o.value === value);
    const delta = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
    if (delta === 0 || index < 0) return;
    event.preventDefault();
    const next = options[(index + delta + options.length) % options.length];
    if (next) {
      onChange(next.value);
      event.currentTarget.querySelector<HTMLElement>(`[data-value="${CSS.escape(next.value)}"]`)?.focus();
    }
  }

  return (
    <div className="ui-segmented" role="radiogroup" aria-label={label} onKeyDown={onKeyDown}>
      {options.map((option) => {
        const checked = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            name={name}
            data-value={option.value}
            aria-checked={checked}
            tabIndex={checked ? 0 : -1}
            className={cx("ui-segmented__option", checked && "ui-segmented__option--checked")}
            onClick={() => onChange(option.value)}
          >
            {option.label}
            {option.count !== undefined && <span className="ui-segmented__count">{option.count}</span>}
          </button>
        );
      })}
    </div>
  );
}
