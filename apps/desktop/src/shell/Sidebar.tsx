import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { cx } from "../components/ui/cx";
import { NAVIGATION, type NavItem, type PageId } from "./navigation";

export interface SidebarProps {
  current: PageId;
  collapsed: boolean;
  onNavigate: (page: PageId) => void;
  onToggleCollapsed: () => void;
}

export function Sidebar({ current, collapsed, onNavigate, onToggleCollapsed }: SidebarProps) {
  return (
    <aside className={cx("shell-sidebar", collapsed && "shell-sidebar--collapsed")}>
      <div className="shell-sidebar__brand">
        <span className="shell-sidebar__logo" aria-hidden="true">
          EX
        </span>
        {!collapsed && <span className="shell-sidebar__name">EVIDENTYX</span>}
      </div>

      <nav className="shell-sidebar__nav" aria-label="Navigation principale">
        {NAVIGATION.map((group) => (
          <div key={group.label} className="shell-nav-group">
            {!collapsed && <p className="shell-nav-group__label section-title">{group.label}</p>}
            <ul>
              {group.items.map((item) => (
                <li key={item.label}>
                  <NavEntry item={item} current={current} collapsed={collapsed} onNavigate={onNavigate} />
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>

      <button
        type="button"
        className="shell-sidebar__toggle"
        onClick={onToggleCollapsed}
        aria-label={collapsed ? "Déplier la barre latérale" : "Replier la barre latérale"}
        title={collapsed ? "Déplier la barre latérale" : "Replier la barre latérale"}
      >
        {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
        {!collapsed && <span>Replier</span>}
      </button>
    </aside>
  );
}

function NavEntry({
  item,
  current,
  collapsed,
  onNavigate,
}: {
  item: NavItem;
  current: PageId;
  collapsed: boolean;
  onNavigate: (page: PageId) => void;
}) {
  const Icon = item.icon;

  if (item.kind === "planned") {
    const hint = `${item.label} — planifiée (${item.plannedFor})`;
    return (
      <span className="shell-nav-item shell-nav-item--planned" aria-disabled="true" title={hint}>
        <Icon size={16} aria-hidden="true" />
        {collapsed ? (
          <span className="sr-only">{hint}</span>
        ) : (
          <>
            <span className="shell-nav-item__label">{item.label}</span>
            <span className="shell-nav-item__planned">planifiée</span>
          </>
        )}
      </span>
    );
  }

  const active = item.page === current;
  return (
    <button
      type="button"
      className={cx("shell-nav-item", active && "shell-nav-item--active")}
      aria-current={active ? "page" : undefined}
      title={collapsed ? item.label : undefined}
      onClick={() => onNavigate(item.page)}
    >
      <Icon size={16} aria-hidden="true" />
      {collapsed ? <span className="sr-only">{item.label}</span> : <span className="shell-nav-item__label">{item.label}</span>}
    </button>
  );
}
