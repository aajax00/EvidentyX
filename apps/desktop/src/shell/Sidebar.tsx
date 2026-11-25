import { FileSearch, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { cx } from "../components/ui/cx";
import { useEngine } from "../engine";
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
  const engine = useEngine();
  const version = engine.kind === "ready" ? `Version ${engine.info.appVersion}` : "Version de démonstration";

  return (
    <aside className={cx("shell-sidebar", collapsed && "shell-sidebar--collapsed")}>
      <div className="shell-sidebar__brand">
        {collapsed ? (
          <BrandMark size={34} />
        ) : (
          <>
            <BrandLogo className="shell-sidebar__logo" />
            <span className="shell-sidebar__wordmark">
              <span className="shell-sidebar__name">EVIDENTYX</span>
              <span className="shell-sidebar__tag">
                Investigation numérique
                <br />& reconstruction d'incidents
              </span>
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

      <div className="shell-sidebar__footer">
        {!collapsed && (
          <div className="shell-sidebar__meta">
            <span>{version}</span>
            <span className="shell-sidebar__oss" title="github.com/aajax00/EvidentyX — licence Apache 2.0">
              <GithubMark /> Projet open source
            </span>
          </div>
        )}
        <button
          type="button"
          className="shell-sidebar__toggle"
          onClick={onToggleCollapsed}
          aria-label={collapsed ? "Déplier la barre latérale" : "Replier la barre latérale"}
          title={collapsed ? "Déplier la barre latérale" : "Replier la barre latérale"}
        >
          {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
        </button>
      </div>
    </aside>
  );
}

/** Marque GitHub (dépôt du projet). */
function GithubMark() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
      <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
    </svg>
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
