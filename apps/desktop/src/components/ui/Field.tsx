import {
  useId,
  type InputHTMLAttributes,
  type ReactNode,
  type TextareaHTMLAttributes,
} from "react";
import { cx } from "./cx";
import "./field.css";

interface FieldChrome {
  label: ReactNode;
  /** Aide affichée sous le champ. */
  hint?: ReactNode;
  /** Message d'erreur : remplace l'aide et marque le champ comme invalide. */
  error?: ReactNode;
}

function useFieldIds(id: string | undefined) {
  const generated = useId();
  const fieldId = id ?? generated;
  return { fieldId, describedBy: `${fieldId}-description` };
}

function FieldFrame({
  fieldId,
  describedBy,
  label,
  hint,
  error,
  required,
  children,
}: FieldChrome & {
  fieldId: string;
  describedBy: string;
  required: boolean | undefined;
  children: ReactNode;
}) {
  const message = error ?? hint;
  return (
    <div className={cx("ui-field", error !== undefined && "ui-field--invalid")}>
      <label htmlFor={fieldId} className="ui-field__label">
        {label}
        {required && (
          <span className="ui-field__required" aria-hidden="true">
            {" "}
            *
          </span>
        )}
      </label>
      {children}
      {message !== undefined && (
        <p id={describedBy} className="ui-field__message" role={error !== undefined ? "alert" : undefined}>
          {message}
        </p>
      )}
    </div>
  );
}

export type TextFieldProps = FieldChrome & Omit<InputHTMLAttributes<HTMLInputElement>, "children">;

export function TextField({ label, hint, error, id, className, required, ...rest }: TextFieldProps) {
  const { fieldId, describedBy } = useFieldIds(id);
  const hasMessage = (error ?? hint) !== undefined;
  return (
    <FieldFrame {...{ fieldId, describedBy, label, hint, error, required }}>
      <input
        {...rest}
        id={fieldId}
        required={required}
        aria-invalid={error !== undefined || undefined}
        aria-describedby={hasMessage ? describedBy : undefined}
        className={cx("ui-input", className)}
      />
    </FieldFrame>
  );
}

export type TextAreaFieldProps = FieldChrome &
  Omit<TextareaHTMLAttributes<HTMLTextAreaElement>, "children">;

export function TextAreaField({
  label,
  hint,
  error,
  id,
  className,
  required,
  rows = 4,
  ...rest
}: TextAreaFieldProps) {
  const { fieldId, describedBy } = useFieldIds(id);
  const hasMessage = (error ?? hint) !== undefined;
  return (
    <FieldFrame {...{ fieldId, describedBy, label, hint, error, required }}>
      <textarea
        {...rest}
        id={fieldId}
        rows={rows}
        required={required}
        aria-invalid={error !== undefined || undefined}
        aria-describedby={hasMessage ? describedBy : undefined}
        className={cx("ui-input", "ui-input--multiline", className)}
      />
    </FieldFrame>
  );
}
