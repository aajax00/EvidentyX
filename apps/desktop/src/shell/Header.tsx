import { ChevronRight, Search } from "lucide-react";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { Kbd } from "./Kbd";
import { SECTION_TITLES } from "./navigation";
import { sectionOf, useRouter } from "./router";

export interface HeaderProps {
  onOpenPalette: () => void;
}

export function Header({ onOpenPalette }: HeaderProps) {
  const { route, navigate } = useRouter();
  const { all } = useInvestigations();
  const section = sectionOf(route);
  const openName =
    route.page === "investigation" ? (all.find((i) => i.id === route.id)?.name ?? `Enquête #${route.id}`) : null;

  return (
    <header className="shell-header">
      <nav className="shell-breadcrumb" aria-label="Fil d'Ariane">
        {openName ? (
          <>
            <button type="button" className="shell-breadcrumb__link" onClick={() => navigate({ page: section })}>
              {SECTION_TITLES[section]}
            </button>
            <ChevronRight size={14} aria-hidden="true" className="shell-breadcrumb__sep" />
            <h1 className="shell-breadcrumb__current" aria-current="page">
              {openName}
            </h1>
          </>
        ) : section === "home" ? (
          <>
            <h1 className="sr-only">{SECTION_TITLES.home}</h1>
            <p className="shell-header__tagline">Chaque trace. Chaque connexion. Chaque vérité.</p>
          </>
        ) : (
          <h1 className="shell-breadcrumb__current">{SECTION_TITLES[section]}</h1>
        )}
      </nav>

      <button type="button" className="shell-header__palette" onClick={onOpenPalette}>
        <Search size={14} aria-hidden="true" />
        <span>Rechercher une commande…</span>
        <Kbd shortcut="mod+k" />
      </button>
    </header>
  );
}
