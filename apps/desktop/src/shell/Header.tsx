import { PanelLeft, Search } from "lucide-react";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { Kbd } from "./Kbd";
import { SECTION_TITLES } from "./navigation";
import { sectionOf, useRouter } from "./router";

export interface HeaderProps {
  onOpenPalette: () => void;
  collapsed: boolean;
  onToggleSidebar: () => void;
}

export function Header({ onOpenPalette, collapsed, onToggleSidebar }: HeaderProps) {
  const { route, navigate } = useRouter();
  const { all } = useInvestigations();
  const section = sectionOf(route);
  const openId = route.page === "investigation" || route.page === "timeline" ? route.id : null;
  const openName = openId === null ? null : (all.find((i) => i.id === openId)?.name ?? `Enquête #${openId}`);

  // Enquête active modifiée le plus récemment : accès direct depuis toute page.
  const latest = all
    .filter((i) => i.status === "active")
    .reduce<(typeof all)[number] | undefined>((best, i) => (!best || i.updatedAt > best.updatedAt ? i : best), undefined);

  return (
    <header className="shell-header">
      <button
        type="button"
        className="shell-header__icon"
        onClick={onToggleSidebar}
        aria-label={collapsed ? "Déplier la barre latérale" : "Replier la barre latérale"}
        title={`${collapsed ? "Déplier" : "Replier"} la barre latérale (Ctrl+B)`}
      >
        <PanelLeft size={16} aria-hidden="true" />
      </button>
      <nav className="shell-breadcrumb" aria-label="Fil d'Ariane">
        <span className="shell-breadcrumb__root">EvidentyX</span>
        <span className="shell-breadcrumb__sep" aria-hidden="true">/</span>
        {openName ? (
          <>
            <button type="button" className="shell-breadcrumb__link" onClick={() => navigate({ page: section })}>
              {SECTION_TITLES[section]}
            </button>
            <span className="shell-breadcrumb__sep" aria-hidden="true">/</span>
            {route.page === "timeline" ? (
              <>
                <button
                  type="button"
                  className="shell-breadcrumb__link"
                  onClick={() => navigate({ page: "investigation", id: route.id })}
                >
                  {openName}
                </button>
                <span className="shell-breadcrumb__sep" aria-hidden="true">/</span>
                <h1 className="shell-breadcrumb__current" aria-current="page">
                  Chronologie
                </h1>
              </>
            ) : (
              <h1 className="shell-breadcrumb__current" aria-current="page">
                {openName}
              </h1>
            )}
          </>
        ) : (
          <h1 className="shell-breadcrumb__current">{SECTION_TITLES[section]}</h1>
        )}
      </nav>

      {latest && latest.id !== openId && (
        <button
          type="button"
          className="shell-header__context"
          title={`Ouvrir l'enquête « ${latest.name} »`}
          onClick={() => navigate({ page: "investigation", id: latest.id })}
        >
          <span>Dernière enquête :</span>
          <span className="shell-header__context-name">{latest.name}</span>
        </button>
      )}
      <button type="button" className="shell-header__palette" onClick={onOpenPalette}>
        <Search size={13} aria-hidden="true" />
        <span>Palette</span>
        <Kbd shortcut="mod+k" />
      </button>
    </header>
  );
}
