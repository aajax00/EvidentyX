import type { ButtonHTMLAttributes, MouseEventHandler, ReactNode } from "react";
import { cx } from "./cx";
import { Spinner } from "./Spinner";
import "./button.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
export type ButtonSize = "sm" | "md";

type BaseProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type" | "onClick"> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Affiche un indicateur et désactive le bouton pendant une action en cours. */
  loading?: boolean;
  children: ReactNode;
};

/**
 * Un bouton a toujours une action : soit un gestionnaire `onClick`,
 * soit `type="submit"` dans un formulaire. Le typage interdit le bouton inerte.
 */
export type ButtonProps = BaseProps &
  (
    | { type?: "button"; onClick: MouseEventHandler<HTMLButtonElement> }
    | { type: "submit" | "reset"; onClick?: MouseEventHandler<HTMLButtonElement> }
  );

export function Button({
  variant = "secondary",
  size = "md",
  loading = false,
  disabled,
  className,
  children,
  type = "button",
  ...rest
}: ButtonProps) {
  return (
    <button
      {...rest}
      type={type}
      disabled={disabled === true || loading}
      aria-busy={loading || undefined}
      className={cx("ui-button", `ui-button--${variant}`, `ui-button--${size}`, className)}
    >
      {loading && <Spinner size="sm" label="Action en cours" />}
      <span className="ui-button__label">{children}</span>
    </button>
  );
}
