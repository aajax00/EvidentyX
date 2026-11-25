// Définition de la navigation principale.
// Une section « planifiée » est affichée mais désactivée : elle n'a pas de page
// et le dit explicitement (règle du projet : aucune page factice).

import {
  FileText,
  FolderSearch,
  House,
  Info,
  Settings,
  ShieldAlert,
  type LucideIcon,
} from "lucide-react";
import type { PageId } from "./router";

export type SectionId = Exclude<PageId, "investigation" | "timeline">;

interface NavItemBase {
  label: string;
  icon: LucideIcon;
}

export interface AvailableNavItem extends NavItemBase {
  kind: "page";
  page: SectionId;
  /** Raccourci clavier éventuel (format de shortcuts.ts). */
  shortcut?: string;
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
    items: [
      { kind: "page", page: "home", label: "Accueil", icon: House },
      { kind: "page", page: "investigations", label: "Enquêtes", icon: FolderSearch },
    ],
  },
  {
    label: "Analyse",
    items: [
      { kind: "planned", label: "Détections", icon: ShieldAlert, plannedFor: "phase 4" },
      { kind: "planned", label: "Rapports", icon: FileText, plannedFor: "phase 4" },
    ],
  },
  {
    label: "Application",
    items: [
      { kind: "page", page: "settings", label: "Paramètres", icon: Settings },
      { kind: "page", page: "about", label: "À propos", icon: Info },
    ],
  },
];

/** Titre de chaque section dans l'en-tête. */
export const SECTION_TITLES: Record<SectionId, string> = {
  home: "Accueil",
  investigations: "Enquêtes",
  settings: "Paramètres",
  about: "À propos",
};

export function availablePages(groups: readonly NavGroup[] = NAVIGATION): AvailableNavItem[] {
  return groups.flatMap((g) => g.items.filter((i): i is AvailableNavItem => i.kind === "page"));
}
