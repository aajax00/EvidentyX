import type { ButtonHTMLAttributes, MouseEventHandler, ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { cx } from "./cx";
import { Spinner } from "./Spinner";
import "./button.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
export type ButtonSize = "sm" | "md";

/** Action obligatoire : un `onClick`, ou `type="submit"` / `"reset"` dans un formulaire. */
export type ButtonAction =
  | { type?: "button"; onClick: MouseEventHandler<HTMLButtonElement> }
  | { type: "submit" | "reset"; onClick?: MouseEventHandler<HTMLButtonElement> };

type BaseProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type" | "onClick"> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Icône placée avant le libellé. */
  icon?: LucideIcon;
  /** Affiche un indicateur et désactive le bouton pendant une action en cours. */
  loading?: boolean;
  children: ReactNode;
};

/**
 * Un bouton a toujours une action : soit un gestionnaire `onClick`,
 * soit `type="submit"` dans un formulaire. Le typage interdit le bouton inerte.
 */
export type ButtonProps = BaseProps & ButtonAction;

export function Button({
  variant = "secondary",
  size = "md",
  icon: Icon,
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
      {loading ? (
        <Spinner size="sm" label="Action en cours" />
      ) : (
        Icon && <Icon size={size === "sm" ? 13 : 14} aria-hidden="true" />
      )}
      <span className="ui-button__label">{children}</span>
    </button>
  );
}

type IconButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type" | "onClick" | "children"> & {
  icon: LucideIcon;
  /** Libellé lu par les lecteurs d'écran et affiché en info-bulle. */
  label: string;
  variant?: "ghost" | "secondary" | "danger";
  size?: ButtonSize;
  onClick: MouseEventHandler<HTMLButtonElement>;
};

/** Bouton réduit à une icône ; son libellé reste accessible (aria-label + info-bulle). */
export function IconButton({
  icon: Icon,
  label,
  variant = "ghost",
  size = "sm",
  className,
  ...rest
}: IconButtonProps) {
  return (
    <button
      {...rest}
      type="button"
      aria-label={label}
      title={label}
      className={cx("ui-button", "ui-icon-button", `ui-button--${variant}`, `ui-button--${size}`, className)}
    >
      <Icon size={size === "sm" ? 14 : 16} aria-hidden="true" />
    </button>
  );
}
