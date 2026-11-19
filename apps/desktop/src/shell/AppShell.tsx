import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { FlaskConical } from "lucide-react";
import { isDemoMode } from "../api";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { CommandPalette } from "./CommandPalette";
import type { Command } from "./commands";
import { Header } from "./Header";
import { availablePages } from "./navigation";
import { useRouter } from "./router";
import { matchesShortcut } from "./shortcuts";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";
import "./shell.css";

/** Nombre d'enquêtes proposées à l'ouverture dans la palette. */
const PALETTE_INVESTIGATIONS = 20;

/**
 * Coque de l'application : barre latérale, en-tête, contenu, barre d'état et
 * palette de commandes. Raccourcis : mod+K (palette), mod+B (barre latérale),
 * mod+N (nouvelle enquête).
 */
export function AppShell({ children }: { children: ReactNode }) {
  const { navigate } = useRouter();
  const { all, openCreate } = useInvestigations();
  const [collapsed, setCollapsed] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);

  const toggleCollapsed = useCallback(() => setCollapsed((c) => !c), []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      // Pas de raccourci global quand une boîte de dialogue est ouverte.
      if (document.querySelector("dialog[open]")) return;
      if (matchesShortcut(event, "mod+k")) {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      } else if (matchesShortcut(event, "mod+b")) {
        event.preventDefault();
        toggleCollapsed();
      } else if (matchesShortcut(event, "mod+n")) {
        event.preventDefault();
        setPaletteOpen(false);
        openCreate();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [toggleCollapsed, openCreate]);

  const commands = useMemo<Command[]>(
    () => [
      {
        id: "new-investigation",
        label: "Nouvelle enquête",
        group: "Enquêtes",
        keywords: ["créer", "ajouter", "dossier"],
        shortcut: "mod+n",
        run: openCreate,
      },
      ...all
        .filter((i) => i.status === "active")
        .slice(0, PALETTE_INVESTIGATIONS)
        .map((inv) => ({
          id: `open-${inv.id}`,
          label: `Ouvrir : ${inv.name}`,
          group: "Enquêtes",
          keywords: ["enquête", inv.description],
          run: () => navigate({ page: "investigation", id: inv.id }),
        })),
      ...availablePages().map((item) => ({
        id: `nav-${item.page}`,
        label: `Aller à : ${item.label}`,
        group: "Navigation",
        keywords: ["page", "afficher"],
        run: () => navigate({ page: item.page }),
      })),
      {
        id: "toggle-sidebar",
        label: collapsed ? "Déplier la barre latérale" : "Replier la barre latérale",
        group: "Affichage",
        keywords: ["menu", "navigation"],
        shortcut: "mod+b",
        run: toggleCollapsed,
      },
    ],
    [all, collapsed, navigate, openCreate, toggleCollapsed],
  );

  return (
    <div className="shell">
      <div className="shell-body">
        <Sidebar collapsed={collapsed} onToggleCollapsed={toggleCollapsed} />
        <div className="shell-main">
          <Header onOpenPalette={() => setPaletteOpen(true)} />
          {isDemoMode() && (
            <p className="shell-demo-banner" role="note">
              <FlaskConical size={14} aria-hidden="true" />
              Mode navigateur : moteur de démonstration en mémoire. Les données affichées sont fictives et ne sont pas
              enregistrées.
            </p>
          )}
          <main className="shell-content">{children}</main>
        </div>
      </div>
      <StatusBar />
      {paletteOpen && <CommandPalette commands={commands} onClose={() => setPaletteOpen(false)} />}
    </div>
  );
}
