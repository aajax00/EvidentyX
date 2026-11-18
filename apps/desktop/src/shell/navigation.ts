// Définition de la navigation principale.
// Une section « planifiée » est affichée mais désactivée : elle n'a pas de page
// et le dit explicitement (règle du projet : aucune page factice).

import {
  Activity,
  FileText,
  FolderSearch,
  Home,
  Info,
  Network,
  Settings,
  ShieldAlert,
  type LucideIcon,
} from "lucide-react";

export type PageId = "home" | "about";

interface NavItemBase {
  label: string;
  icon: LucideIcon;
}

export interface AvailableNavItem extends NavItemBase {
  kind: "page";
  page: PageId;
}

export interface PlannedNavItem extends NavItemBase {
  kind: "planned";
  /** Étape ou phase de la feuille de route qui livrera la section. */
  plannedFor: string;
}

export type NavItem = AvailableNavItem | PlannedNavItem;

export interface NavGroup {
  label: string;
  items: readonly NavItem[];
}

export const NAVIGATION: readonly NavGroup[] = [
  {
    label: "Général",
    items: [{ kind: "page", page: "home", label: "Accueil", icon: Home }],
  },
  {
    label: "Enquête",
    items: [
      { kind: "planned", label: "Enquêtes", icon: FolderSearch, plannedFor: "étape 1.6" },
      { kind: "planned", label: "Chronologie", icon: Activity, plannedFor: "phase 3" },
      { kind: "planned", label: "Graphe", icon: Network, plannedFor: "phase 3" },
      { kind: "planned", label: "Détections", icon: ShieldAlert, plannedFor: "phase 4" },
      { kind: "planned", label: "Rapports", icon: FileText, plannedFor: "phase 4" },
    ],
  },
  {
    label: "Application",
    items: [
      { kind: "planned", label: "Paramètres", icon: Settings, plannedFor: "étape 1.7" },
      { kind: "page", page: "about", label: "À propos", icon: Info },
    ],
  },
];

/** Titre affiché dans l'en-tête pour chaque page. */
export const PAGE_TITLES: Record<PageId, string> = {
  home: "Accueil",
  about: "À propos",
};

export function availablePages(groups: readonly NavGroup[] = NAVIGATION): AvailableNavItem[] {
  return groups.flatMap((g) => g.items.filter((i): i is AvailableNavItem => i.kind === "page"));
}
