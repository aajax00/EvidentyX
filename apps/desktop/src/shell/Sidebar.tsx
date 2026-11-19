import { FileSearch, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { cx } from "../components/ui/cx";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { BrandLogo, BrandMark } from "./BrandLogo";
import { NAVIGATION, type NavItem, type SectionId } from "./navigation";
import { sectionOf, useRouter } from "./router";

export interface SidebarProps {
  collapsed: boolean;
  onToggleCollapsed: () => void;
}

/** Nombre d'enquêtes récentes listées sous la navigation. */
const RECENT_COUNT = 5;

export function Sidebar({ collapsed, onToggleCollapsed }: SidebarProps) {
  const { route, navigate } = useRouter();
  const { all } = useInvestigations();
  const current = sectionOf(route);
  const openId = route.page === "investigation" ? route.id : null;
  const recent = all.filter((i) => i.status === "active").slice(0, RECENT_COUNT);

  return (
    <aside className={cx("shell-sidebar", collapsed && "shell-sidebar--collapsed")}>
      <div className="shell-sidebar__brand">
        {collapsed ? (
          <BrandMark size={30} />
        ) : (
          <>
            <BrandLogo className="shell-sidebar__logo" />
            <span className="shell-sidebar__wordmark">
              <span className="shell-sidebar__name">EVIDENTYX</span>
              <span className="shell-sidebar__tag">Investigation numérique</span>
            </span>
          </>
        )}
      </div>

      <nav className="shell-sidebar__nav" aria-label="Navigation principale">
        {NAVIGATION.map((group) => (
          <div key={group.label} className="shell-nav-group">
            {!collapsed && <p className="shell-nav-group__label section-title">{group.label}</p>}
            <ul>
              {group.items.map((item) => (
                <li key={item.label}>
                  <NavEntry
                    item={item}
                    current={current}
                    collapsed={collapsed}
                    onNavigate={(page) => navigate({ page })}
                  />
                </li>
              ))}
            </ul>
          </div>
        ))}

        {!collapsed && recent.length > 0 && (
          <div className="shell-nav-group">
            <p className="shell-nav-group__label section-title">Récentes</p>
            <ul>
              {recent.map((inv) => (
                <li key={inv.id}>
                  <button
                    type="button"
                    className={cx("shell-nav-item", "shell-nav-item--recent", openId === inv.id && "shell-nav-item--active")}
                    aria-current={openId === inv.id ? "page" : undefined}
                    title={inv.name}
                    onClick={() => navigate({ page: "investigation", id: inv.id })}
                  >
                    <FileSearch size={14} aria-hidden="true" />
                    <span className="shell-nav-item__label">{inv.name}</span>
                  </button>
                </li>
              ))}
            </ul>
          </div>
        )}
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
  current: SectionId;
  collapsed: boolean;
  onNavigate: (page: SectionId) => void;
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
