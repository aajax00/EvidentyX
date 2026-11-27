import { FileSearch } from "lucide-react";
import { cx } from "../components/ui/cx";
import { useEngine } from "../engine";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { BrandMark } from "./BrandLogo";
import { NAVIGATION, type NavItem, type SectionId } from "./navigation";
import { sectionOf, useRouter } from "./router";

export interface SidebarProps {
  collapsed: boolean;
}

/** Nombre d'enquêtes récentes listées sous la navigation. */
const RECENT_COUNT = 5;

export function Sidebar({ collapsed }: SidebarProps) {
  const { route, navigate } = useRouter();
  const { all } = useInvestigations();
  const current = sectionOf(route);
  const openId = route.page === "investigation" || route.page === "timeline" ? route.id : null;
  const active = all.filter((i) => i.status === "active");
  const recent = active.slice(0, RECENT_COUNT);
  const engine = useEngine();

  return (
    <aside className={cx("shell-sidebar", collapsed && "shell-sidebar--collapsed")}>
      <button
        type="button"
        className="shell-sidebar__brand"
        title="EvidentyX — Tableau de bord"
        onClick={() => navigate({ page: "home" })}
      >
        <BrandMark size={24} />
        {!collapsed && (
          <span className="shell-sidebar__wordmark">
            Evidenty<span className="shell-sidebar__x">X</span>
          </span>
        )}
      </button>

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
                    count={item.kind === "page" && item.page === "investigations" ? active.length : undefined}
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

      {!collapsed && (
        <div className="shell-sidebar__footer mono">
          {engine.kind === "ready" ? `EvidentyX v${engine.info.appVersion}` : "EvidentyX"}
          <span>Hors ligne · aucune télémétrie</span>
        </div>
      )}
    </aside>
  );
}

function NavEntry({
  item,
  current,
  collapsed,
  count,
  onNavigate,
}: {
  item: NavItem;
  current: SectionId;
  collapsed: boolean;
  /** Compteur affiché à droite du libellé (enquêtes actives). */
  count?: number | undefined;
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
      {!collapsed && count !== undefined && <span className="shell-nav-item__count mono">{count}</span>}
    </button>
  );
}
