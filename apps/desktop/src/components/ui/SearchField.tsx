import { forwardRef, type InputHTMLAttributes } from "react";
import { Search, X } from "lucide-react";
import "./field.css";

export type SearchFieldProps = Omit<InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "type"> & {
  /** Libellé lu par les lecteurs d'écran. */
  label: string;
  value: string;
  onChange: (value: string) => void;
};

/** Champ de recherche avec icône et bouton d'effacement. */
export const SearchField = forwardRef<HTMLInputElement, SearchFieldProps>(function SearchField(
  { label, value, onChange, ...rest },
  ref,
) {
  return (
    <span className="ui-search">
      <Search size={14} aria-hidden="true" className="ui-search__icon" />
      <input
        {...rest}
        ref={ref}
        type="search"
        aria-label={label}
        value={value}
        spellCheck={false}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape" && value) {
            e.preventDefault();
            e.stopPropagation();
            onChange("");
          }
          rest.onKeyDown?.(e);
        }}
        className="ui-input ui-search__input"
      />
      {value && (
        <button type="button" className="ui-search__clear" aria-label="Effacer la recherche" title="Effacer" onClick={() => onChange("")}>
          <X size={13} aria-hidden="true" />
        </button>
      )}
    </span>
  );
});
