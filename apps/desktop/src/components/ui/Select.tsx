import { useId, type SelectHTMLAttributes } from "react";
import { ChevronDown } from "lucide-react";
import { cx } from "./cx";
import "./field.css";

export interface SelectOption<T extends string> {
  value: T;
  label: string;
}

export type SelectProps<T extends string> = Omit<SelectHTMLAttributes<HTMLSelectElement>, "value" | "onChange" | "children"> & {
  /** Libellé visible placé avant la liste. */
  label: string;
  value: T;
  options: readonly SelectOption<T>[];
  onChange: (value: T) => void;
};

/** Liste déroulante native stylée, avec libellé en ligne. */
export function Select<T extends string>({ label, value, options, onChange, className, id, ...rest }: SelectProps<T>) {
  const generated = useId();
  const selectId = id ?? generated;
  return (
    <div className="ui-select">
      <label htmlFor={selectId} className="ui-select__label">
        {label}
      </label>
      <span className="ui-select__control">
        <select
          {...rest}
          id={selectId}
          value={value}
          onChange={(e) => onChange(e.target.value as T)}
          className={cx("ui-input", "ui-select__input", className)}
        >
          {options.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
        <ChevronDown size={14} aria-hidden="true" className="ui-select__chevron" />
      </span>
    </div>
  );
}
